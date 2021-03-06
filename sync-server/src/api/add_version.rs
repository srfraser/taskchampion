use crate::api::{
    failure_to_ise, ServerState, HISTORY_SEGMENT_CONTENT_TYPE, PARENT_VERSION_ID_HEADER,
    VERSION_ID_HEADER,
};
use crate::server::{add_version, AddVersionResult, ClientId, VersionId, NO_VERSION_ID};
use actix_web::{error, post, web, HttpMessage, HttpRequest, HttpResponse, Result};
use futures::StreamExt;

/// Max history segment size: 100MB
const MAX_SIZE: usize = 100 * 1024 * 1024;

/// Add a new version, after checking prerequisites.  The history segment should be transmitted in
/// the request entity body and must have content-type
/// `application/vnd.taskchampion.history-segment`.  The content can be encoded in any of the
/// formats supported by actix-web.
///
/// On success, the response is a 200 OK with the new version ID in the `X-Version-Id` header.  If
/// the version cannot be added due to a conflict, the response is a 409 CONFLICT with the expected
/// parent version ID in the `X-Parent-Version-Id` header.
///
/// Returns other 4xx or 5xx responses on other errors.
#[post("/client/{client_id}/add-version/{parent_version_id}")]
pub(crate) async fn service(
    req: HttpRequest,
    server_state: web::Data<ServerState>,
    web::Path((client_id, parent_version_id)): web::Path<(ClientId, VersionId)>,
    mut payload: web::Payload,
) -> Result<HttpResponse> {
    // check content-type
    if req.content_type() != HISTORY_SEGMENT_CONTENT_TYPE {
        return Err(error::ErrorBadRequest("Bad content-type"));
    }

    // read the body in its entirety
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    if body.is_empty() {
        return Err(error::ErrorBadRequest("Empty body"));
    }

    // note that we do not open the transaction until the body has been read
    // completely, to avoid blocking other storage access while that data is
    // in transit.
    let mut txn = server_state.txn().map_err(failure_to_ise)?;

    // get, or create, the client
    let client = match txn.get_client(client_id).map_err(failure_to_ise)? {
        Some(client) => client,
        None => {
            txn.new_client(client_id, NO_VERSION_ID)
                .map_err(failure_to_ise)?;
            txn.get_client(client_id).map_err(failure_to_ise)?.unwrap()
        }
    };

    let result = add_version(txn, client_id, client, parent_version_id, body.to_vec())
        .map_err(failure_to_ise)?;
    Ok(match result {
        AddVersionResult::Ok(version_id) => HttpResponse::Ok()
            .header(VERSION_ID_HEADER, version_id.to_string())
            .body(""),
        AddVersionResult::ExpectedParentVersion(parent_version_id) => HttpResponse::Conflict()
            .header(PARENT_VERSION_ID_HEADER, parent_version_id.to_string())
            .body(""),
    })
}

#[cfg(test)]
mod test {
    use crate::api::ServerState;
    use crate::app_scope;
    use crate::storage::{InMemoryStorage, Storage};
    use actix_web::{http::StatusCode, test, App};
    use uuid::Uuid;

    #[actix_rt::test]
    async fn test_success() {
        let client_id = Uuid::new_v4();
        let version_id = Uuid::new_v4();
        let parent_version_id = Uuid::new_v4();
        let server_box: Box<dyn Storage> = Box::new(InMemoryStorage::new());

        // set up the storage contents..
        {
            let mut txn = server_box.txn().unwrap();
            txn.new_client(client_id, Uuid::nil()).unwrap();
        }

        let server_state = ServerState::new(server_box);
        let mut app = test::init_service(App::new().service(app_scope(server_state))).await;

        let uri = format!("/client/{}/add-version/{}", client_id, parent_version_id);
        let req = test::TestRequest::post()
            .uri(&uri)
            .header(
                "Content-Type",
                "application/vnd.taskchampion.history-segment",
            )
            .set_payload(b"abcd".to_vec())
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        // the returned version ID is random, but let's check that it's not
        // the passed parent version ID, at least
        let new_version_id = resp.headers().get("X-Version-Id").unwrap();
        assert!(new_version_id != &version_id.to_string());

        assert_eq!(resp.headers().get("X-Parent-Version-Id"), None);
    }

    #[actix_rt::test]
    async fn test_conflict() {
        let client_id = Uuid::new_v4();
        let version_id = Uuid::new_v4();
        let parent_version_id = Uuid::new_v4();
        let server_box: Box<dyn Storage> = Box::new(InMemoryStorage::new());

        // set up the storage contents..
        {
            let mut txn = server_box.txn().unwrap();
            txn.new_client(client_id, version_id).unwrap();
        }

        let server_state = ServerState::new(server_box);
        let mut app = test::init_service(App::new().service(app_scope(server_state))).await;

        let uri = format!("/client/{}/add-version/{}", client_id, parent_version_id);
        let req = test::TestRequest::post()
            .uri(&uri)
            .header(
                "Content-Type",
                "application/vnd.taskchampion.history-segment",
            )
            .set_payload(b"abcd".to_vec())
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::CONFLICT);
        assert_eq!(resp.headers().get("X-Version-Id"), None);
        assert_eq!(
            resp.headers().get("X-Parent-Version-Id").unwrap(),
            &version_id.to_string()
        );
    }

    #[actix_rt::test]
    async fn test_bad_content_type() {
        let client_id = Uuid::new_v4();
        let parent_version_id = Uuid::new_v4();
        let server_box: Box<dyn Storage> = Box::new(InMemoryStorage::new());
        let server_state = ServerState::new(server_box);
        let mut app = test::init_service(App::new().service(app_scope(server_state))).await;

        let uri = format!("/client/{}/add-version/{}", client_id, parent_version_id);
        let req = test::TestRequest::post()
            .uri(&uri)
            .header("Content-Type", "not/correct")
            .set_payload(b"abcd".to_vec())
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_rt::test]
    async fn test_empty_body() {
        let client_id = Uuid::new_v4();
        let parent_version_id = Uuid::new_v4();
        let server_box: Box<dyn Storage> = Box::new(InMemoryStorage::new());
        let server_state = ServerState::new(server_box);
        let mut app = test::init_service(App::new().service(app_scope(server_state))).await;

        let uri = format!("/client/{}/add-version/{}", client_id, parent_version_id);
        let req = test::TestRequest::post()
            .uri(&uri)
            .header(
                "Content-Type",
                "application/vnd.taskchampion.history-segment",
            )
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
}
