use ractor::rpc::CallResult;
use ractor::{Actor,ActorProcessingErr,ActorRef,RpcReplyPort};
use std::time::{Duration,Instant};
use tokio::time::sleep;

#[derive(Debug)]
pub enum CounterMessage{
    Increment {amount:i64},
    Decrement {amount:i64},
    GetValue(RpcReplyPort<i64>),
    GetStats(RpcReplyPort<CounterStats>)
}

#[derive(Debug,Clone)]
pub struct CounterStats{
    pub current_value:i64,
    pub total_operations: u64,
    pub operations_per_second: f64,
}

pub struct CounterActor{
    id:String,
}

pub struct CounterState{
    id: String,
    value:i64,
    total_operations:u64,
    start_time: Instant,
}

#[ractor::async_trait]
impl Actor for CounterActor{
    type Msg = CounterMessage;
    type State = CounterState;
    type Arguments = String;

    async fn pre_start(&self, _myself: ActorRef<Self::Msg>, id: Self::Arguments) -> Result<Self::State, ActorProcessingErr> {
        println!("[{} Counter starting",id);
        Ok(CounterState{
            id,
            value:0,
            total_operations:0,
            start_time: Instant::now(),
        })
    }

    async fn post_stop(&self, _myself: ActorRef<Self::Msg>, state: &mut Self::State) -> Result<(), ActorProcessingErr> {
        println!("[{}] Counter stopping with final values {} after {} operations",state.id,state.value, state.total_operations);
        Ok(())
    }

    async fn handle(&self, _myself: ActorRef<Self::Msg>, message: Self::Msg, state: &mut Self::State) -> Result<(), ActorProcessingErr> {
        match message{
            CounterMessage::Increment {amount} => {
                state.value += amount;
                state.total_operations += 1;
            }
            CounterMessage::Decrement {amount} => {
                state.value -= amount;
                state.total_operations -= 1;
            }
            CounterMessage::GetValue(reply) => {
                reply.send(state.value)?;
            }
            CounterMessage::GetStats(reply) => {
                let elapsed = state.start_time.elapsed().as_secs_f64();
                let ops_per_second = if elapsed>0.0{
                    state.total_operations as f64/elapsed
                }
                else{
                    0.0
                };
                reply.send(CounterStats{
                    current_value:state.value,
                    total_operations: state.total_operations,
                    operations_per_second: ops_per_second,
                })?;
            }
        }
        Ok(())
    }
}

pub enum AggregatorMessage{
    RegisterCounter(ActorRef<CounterMessage>),
    CollectStats,
    GetTotalStats(RpcReplyPort<AggregatedStats>)
}

#[derive(Debug,Clone)]
pub struct AggregatedStats{
    pub total_value:i64,
    pub total_operations:u64,
    pub num_counters:usize,
    pub avg_ops_per_second:f64,
}

pub struct AggregatorActor;

pub struct AggregatorState{
    counters:Vec<ActorRef<CounterMessage>>,
    last_stats: Option<AggregatedStats>,
}

#[ractor::async_trait]
impl Actor for AggregatorActor{
    type Msg = AggregatorMessage;
    type State = AggregatorState;
    type Arguments = ();

    async fn pre_start(&self, myself: ActorRef<Self::Msg>, args: Self::Arguments) -> Result<Self::State, ActorProcessingErr> {
        println!("[Aggregator] starting");
        Ok(AggregatorState{
            counters:Vec::new(),
            last_stats: None,
        })
    }

    async fn handle(&self, myself: ActorRef<Self::Msg>, message: Self::Msg, state: &mut Self::State) -> Result<(), ActorProcessingErr> {
        match message {
            AggregatorMessage::RegisterCounter(reply) => {
                println!("[Aggregator] registering counter");
            }
            AggregatorMessage::CollectStats => {
                let mut total_value=0i64;
                let mut total_operations=0u64;
                let mut total_ops_per_second=0.0;

                for counter in &state.counters{
                    match counter.call(
                        |reply|CounterMessage::GetStats(reply),
                        Some(Duration::from_secs(1))
                    ).await{
                        Ok(CallResult::Success(stats))=>{
                            total_value+=stats.current_value;
                            total_operations+=stats.total_operations;
                            total_ops_per_second+=stats.operations_per_second;
                        }
                        Ok(CallResult::Timeout)=>{
                            eprintln!("[Aggregator] timed out");
                        }
                        Ok(CallResult::SenderError)=>{
                            eprintln!("[Aggregator] sender error");
                        }
                        Err(e)=>{
                            eprintln!("[Aggregator] failed to get stats: {:?}", e);
                        }
                    }
                }
                let aggregated=AggregatedStats{
                    total_value,
                    total_operations,
                    num_counters: state.counters.len(),
                    avg_ops_per_second: if state.counters.len()>0{
                        total_ops_per_second/state.counters.len() as f64
                    }else{
                        0.0
                    }
                };
                println!("[Aggregator] Toal value: {}, Total ops: {}, Abgops/sec: {:.2}",
                         aggregated.total_value,aggregated.total_operations,aggregated.avg_ops_per_second);
                state.last_stats=Some(aggregated);

                //schedule next collection
                tokio::spawn(async move{
                    tokio::time::sleep(Duration.from_secs(2)).await;
                    let _=myself.cast(AggregatorMessage::CollectStats);
                })
            }
            AggregatorMessage::GetTotalStats(reply) => {
                if let Some(stats)=&state.last_stats {
                    reply.send(stats.clone())?;
                }else{
                    reply.send(AggregatedStats{
                        total_value:0,
                        total_operations:0,
                        num_counters:0,
                        avg_ops_per_second:0.0
                    })?;
                }
            }
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() {

}