use heed::Env;
use tonic::{transport::Server, Request, Response, Status};

use crate::db::{env as dbenv, kv, txn::Txn};

pub mod proto {
    tonic::include_proto!("automation");
}
use proto::automation_server::{Automation, AutomationServer};
use proto::{
    DeleteRequest, DeleteResponse, GetRequest, GetResponse, ListDatabasesRequest,
    ListDatabasesResponse, PutRequest, PutResponse,
};

pub struct AutomationService {
    env: Env,
}

impl AutomationService {
    pub fn new(env: Env) -> Self {
        Self { env }
    }
}

#[tonic::async_trait]
impl Automation for AutomationService {
    async fn list_databases(
        &self,
        _req: Request<ListDatabasesRequest>,
    ) -> Result<Response<ListDatabasesResponse>, Status> {
        let names =
            dbenv::list_databases(&self.env).map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ListDatabasesResponse { names }))
    }

    async fn get(&self, req: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        let req = req.into_inner();
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| Status::internal(e.to_string()))?;
        let db: heed::Database<heed::types::Str, heed::types::Bytes> = self
            .env
            .open_database(&rtxn, Some(&req.db))
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("database not found"))?;
        let value = db
            .get(&rtxn, &req.key)
            .map_err(|e| Status::internal(e.to_string()))?
            .map(|v| v.to_vec());
        match value {
            Some(v) => Ok(Response::new(GetResponse {
                value: v,
                found: true,
            })),
            None => Ok(Response::new(GetResponse {
                value: Vec::new(),
                found: false,
            })),
        }
    }

    async fn put(&self, req: Request<PutRequest>) -> Result<Response<PutResponse>, Status> {
        let req = req.into_inner();
        let mut txn = Txn::begin(&self.env).map_err(|e| Status::internal(e.to_string()))?;
        match kv::put(&self.env, &mut txn, &req.db, &req.key, &req.value) {
            Ok(()) => txn.commit().map_err(|e| Status::internal(e.to_string()))?,
            Err(e) => {
                txn.abort();
                return Err(Status::internal(e.to_string()));
            }
        }
        Ok(Response::new(PutResponse {}))
    }

    async fn delete(
        &self,
        req: Request<DeleteRequest>,
    ) -> Result<Response<DeleteResponse>, Status> {
        let req = req.into_inner();
        let mut txn = Txn::begin(&self.env).map_err(|e| Status::internal(e.to_string()))?;
        match kv::delete(&self.env, &mut txn, &req.db, &req.key) {
            Ok(()) => txn.commit().map_err(|e| Status::internal(e.to_string()))?,
            Err(e) => {
                txn.abort();
                return Err(Status::internal(e.to_string()));
            }
        }
        Ok(Response::new(DeleteResponse {}))
    }
}

pub async fn serve(env: Env, addr: std::net::SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let svc = AutomationService::new(env);
    Server::builder()
        .add_service(AutomationServer::new(svc))
        .serve(addr)
        .await?;
    Ok(())
}
