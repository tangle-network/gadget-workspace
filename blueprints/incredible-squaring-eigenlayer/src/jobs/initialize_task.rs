use crate::IIncredibleSquaringTaskManager::Task;
use crate::{
    contexts::aggregator::AggregatorContext, Error, IncredibleSquaringTaskManager, ProcessorError,
    INCREDIBLE_SQUARING_TASK_MANAGER_ABI_STRING,
};
use gadget_event_listeners::evm::EvmContractEventListener;
use gadget_logging::info;
use gadget_macros::job;
use gadget_std::{convert::Infallible, ops::Deref};

const TASK_CHALLENGE_WINDOW_BLOCK: u32 = 100;
const BLOCK_TIME_SECONDS: u32 = 12;

/// Initializes the task for the aggregator server
#[job(
    id = 1,
    params(task, task_index),
    event_listener(
        listener = EvmContractEventListener<IncredibleSquaringTaskManager::NewTaskCreated>,
        instance = IncredibleSquaringTaskManager,
        abi = INCREDIBLE_SQUARING_TASK_MANAGER_ABI_STRING,
        pre_processor = convert_event_to_inputs,
    ),
)]
pub async fn initialize_bls_task(
    ctx: AggregatorContext,
    task: Task,
    task_index: u32,
) -> Result<u32, Infallible> {
    info!("Initializing task for BLS aggregation");

    let mut tasks = ctx.tasks.lock().await;
    tasks.insert(task_index, task.clone());
    let time_to_expiry =
        std::time::Duration::from_secs((TASK_CHALLENGE_WINDOW_BLOCK * BLOCK_TIME_SECONDS).into());

    if let Some(service) = &ctx.bls_aggregation_service {
        service
            .lock()
            .await
            .initialize_new_task(
                task_index,
                task.taskCreatedBlock,
                task.quorumNumbers.to_vec(),
                vec![task.quorumThresholdPercentage.try_into().unwrap(); task.quorumNumbers.len()],
                time_to_expiry,
            )
            .await
            .unwrap()
    }

    Ok(1)
}

/// Converts the event to inputs.
///
/// Uses a tuple to represent the return type because
/// the macro will index all values in the #[job] function
/// and parse the return type by the index.
pub async fn convert_event_to_inputs(
    event: (
        IncredibleSquaringTaskManager::NewTaskCreated,
        alloy_rpc_types::Log,
    ),
) -> Result<Option<(Task, u32)>, ProcessorError> {
    let task_index = event.0.taskIndex;
    Ok(Some((event.0.task, task_index)))
}
