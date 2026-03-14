use std::any::Any;
use aspect_core::{Aspect, AspectError, AsyncAspect, AsyncProceedingJoinPoint, ProceedingJoinPoint};
use tokio::time::Instant;
use tracing::info;

#[derive(Default)]
pub struct Timer;

impl AsyncAspect for Timer {
    async fn around(
        &self,
        pjp: AsyncProceedingJoinPoint<'_>,
    ) ->  Result<Box<dyn Any + Send + Sync>, AspectError>  {
        let start = Instant::now();
        let function_name = pjp.context().function_name;
        let result = pjp.proceed().await;
        let elapsed = start.elapsed();
        info!("{} took {:?}", function_name, elapsed);
        result
    }
}


// impl Aspect for Timer {
//     fn around(&self, pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError>  {
//         let start = Instant::now();
//         let function_name = pjp.context().function_name;
//         let result = pjp.proceed();
//         let elapsed = start.elapsed();
//         info!("{} took {:?}", function_name, elapsed);
//         result
//     }
// }
