use std::any::Any;
use std::sync::Arc;
use aspect_core::{AspectError, AsyncAspect, AsyncJoinPoint, AsyncProceedingJoinPoint};
use tokio::time::Instant;
use tracing::info;

#[derive(Default)]
pub struct Logger;
impl AsyncAspect for Logger {
    async fn before(&self, ctx: &AsyncJoinPoint) {
        // let arg0 = ctx.args.get(0).and_then(|b| b.downcast_ref::<Arc<AppState>>());
        // if let Some(app_state) = arg0 {
        //     info!("Logger.before: received AppState (rbatis pool present)");
        // } else {
        //     info!("Logger.before: arg0 missing or not Arc<AppState>");
        // }
        //
        // let arg1 = ctx.args.get(1).and_then(|b| b.downcast_ref::<QueryUserListReq>());
        // if let Some(q) = arg1 {
        //     info!("Logger.before: page_no = {}", q.page_no);
        //     info!("{function_name}:{q:?}",function_name = ctx.function_name);
        // } else {
        //     info!("Logger.before: arg1 missing or not QueryUserListReq");
        // }
        // info!("{}: {},{},{},{}", ctx.function_name, ctx.module_path, ctx.location.file, ctx.location.line, ctx.args.iter().count());
    }

    async fn after(&self, _ctx: &AsyncJoinPoint, _result: &(dyn Any + Send + Sync))  {
        // info!("Logger.after: function completed");
    }

    // async fn around(&self, pjp: AsyncProceedingJoinPoint<'_>) -> Result<Box<dyn Any + Send + Sync>, AspectError> {
    //     let start = Instant::now();
    //     let function_name = pjp.context().function_name;
    //     info!("Logger.around enter: {}", function_name);
    //     let result = pjp.proceed().await;
    //     let elapsed = start.elapsed();
    //     info!("{} took {:?}", function_name, elapsed);
    //     match &result {
    //         Ok(_) => info!("{} executed successfully", function_name),
    //         Err(e) => info!("{} execution failed: {:?}", function_name, e),
    //     };
    //     result
    // }
}
