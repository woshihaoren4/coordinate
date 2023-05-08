use std::sync::Arc;
use std::task::{Context, Poll};

#[tonic::async_trait]
pub trait LayerHyperInterceptor: Send + Sync {
    async fn request(
        &self,
        _ctx: wd_event::Context,
        request: hyper::Request<hyper::Body>,
    ) -> Result<hyper::Request<hyper::Body>, hyper::Response<tonic::body::BoxBody>> {
        return Ok(request);
    }
    async fn response(
        &self,
        _ctx: wd_event::Context,
        response: hyper::Response<tonic::body::BoxBody>,
    ) -> hyper::Response<tonic::body::BoxBody> {
        return response;
    }
}

#[derive(Clone)]
pub struct CustomInterceptor {
    inner: Arc<dyn LayerHyperInterceptor + 'static>,
}

impl CustomInterceptor {
    pub fn new<I>(inner: I) -> Self
        where
            I: LayerHyperInterceptor + 'static,
    {
        let inner = Arc::new(inner);
        return Self { inner };
    }
}

impl<S> tower::Layer<S> for CustomInterceptor {
    type Service = MyMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        MyMiddleware {
            inner: service,
            handle: self.inner.clone(),
        }
    }
}

#[derive(Clone)]
pub struct MyMiddleware<S> {
    inner: S,
    handle: Arc<dyn LayerHyperInterceptor + 'static>,
}

impl<S> tower::Service<hyper::Request<hyper::Body>> for MyMiddleware<S>
    where
        S: tower::Service<
            hyper::Request<hyper::Body>,
            Response = hyper::Response<tonic::body::BoxBody>,
        > + Clone
        + Send
        + 'static,
        S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = futures::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: hyper::Request<hyper::Body>) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        let handle = self.handle.clone();
        Box::pin(async move {
            let ctx = wd_event::Context::default();
            let result = handle.request(ctx.clone(), req).await;
            let req = match result {
                Ok(req) => req,
                Err(response) => {
                    return Ok(response);
                }
            };
            let response = inner.call(req).await?;
            let response = handle.response(ctx, response).await;
            Ok(response)
        })
    }
}