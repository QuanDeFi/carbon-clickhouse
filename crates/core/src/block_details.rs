use crate::datasource::BlockDetails;
use crate::error::CarbonResult;
use crate::filter::Filter;
use crate::processor::Processor;
use async_trait::async_trait;

pub struct BlockDetailsPipe<P> {
    pub processor: P,
    pub filters: Vec<Box<dyn Filter + Send + Sync + 'static>>,
}

#[async_trait]
pub trait BlockDetailsPipes: Send + Sync {
    async fn run(&mut self, block_details: BlockDetails) -> CarbonResult<()>;
    async fn finalize(&mut self) -> CarbonResult<()>;

    fn filters(&self) -> &Vec<Box<dyn Filter + Send + Sync + 'static>>;
}

#[async_trait]
impl<P> BlockDetailsPipes for BlockDetailsPipe<P>
where
    P: Processor<BlockDetails> + Send + Sync,
{
    async fn run(&mut self, block_details: BlockDetails) -> CarbonResult<()> {
        self.processor.process(&block_details).await?;

        Ok(())
    }

    async fn finalize(&mut self) -> CarbonResult<()> {
        self.processor.finalize().await
    }

    fn filters(&self) -> &Vec<Box<dyn Filter + Send + Sync + 'static>> {
        &self.filters
    }
}
