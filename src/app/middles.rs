use hyper::{Body, Response};
use tonic::body::BoxBody;
use tonic::{Code, Status};
use wd_event::Context;
use wd_tools::{ PFErr, PFOk};
use crate::infra::exit::Exit;
use crate::infra::middle::LayerHyperInterceptor;

#[derive(Debug)]
pub struct MiddleExit{
    exit:Exit,
}

impl MiddleExit {
    pub fn new(exit:Exit)->MiddleExit{
        MiddleExit{exit}
    }
}

#[tonic::async_trait]
impl LayerHyperInterceptor for MiddleExit {
    async fn request(&self, _ctx: Context, request: hyper::Request<Body>) -> Result<hyper::Request<Body>, Response<BoxBody>> {
        match self.exit.status() {
            0=> return Status::new(Code::Unavailable,"server not ready").to_http().err(),
            1=> self.exit.add_task(),
            2=> return Status::new(Code::Unavailable,"server exiting").to_http().err(),
            _=> return Status::new(Code::Unavailable,"server abnormal").to_http().err(),
        };
        request.ok()
    }

    async fn response(&self, _ctx: Context, response: Response<BoxBody>) -> Response<BoxBody> {
        self.exit.sub_task();
        response
    }
}


pub struct MiddleLog;

impl MiddleLog {
    const REQUEST_ID: &'static str = "MiddleLog_req_id";
    async fn set_request_id(ctx:&mut Context)->String{
        let req_id = wd_tools::uuid::v4();
        ctx.store(Self::REQUEST_ID,req_id.clone()).await;
        req_id
    }
    async fn get_request_id(ctx:&mut Context)->String{
        ctx.def_function(Self::REQUEST_ID,|x:Option<&String>|{
            x.unwrap_or(&("none".to_string())).to_string()
        }).await
    }
}

#[tonic::async_trait]
impl LayerHyperInterceptor for MiddleLog {
    async fn request(&self, mut ctx: Context, request: hyper::Request<Body>) -> Result<hyper::Request<Body>, Response<BoxBody>> {
        let rid = MiddleLog::set_request_id(&mut ctx).await;
        let path = request.uri().path();
        wd_log::log_debug_ln!("request[{}]---> method:{}",rid,path);
        request.ok()
    }

    async fn response(&self, mut ctx: Context, response: Response<BoxBody>) -> Response<BoxBody> {
        let rid = MiddleLog::get_request_id(&mut ctx).await;
        let grpc_result = match response.headers().get("grpc-status") {
            Some(s)=> s.to_str().unwrap_or("2"),
            None=>"2",
        };
        if grpc_result == "2" {
            wd_log::log_debug_ln!("response[{}]---> ok",rid);
        }else{
            let grpc_message = match response.headers().get("grpc-message") {
                Some(s)=>s.to_str().unwrap_or("none"),
                None=>"none",
            };
            wd_log::log_debug_ln!("response[{}] error---> code:{} error:{}",rid,grpc_result,grpc_message);
        }

        response
    }
}