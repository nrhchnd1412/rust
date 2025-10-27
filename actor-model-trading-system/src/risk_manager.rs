use ractor::{ActorRef, Actor, ActorProcessingErr, SupervisionEvent, RpcReplyPort};
use std::time::Duration;
use tokio::time::sleep;
use crate::order_book;
use crate::executor;
use crate::executor::ExecutorMessage;

pub enum RiskMessage{
    CheckOrder(order_book::Order),
    GetExposure(ractor::RpcReplyPort<f64>),
    Reset,
}
pub struct RiskManager;

pub struct RiskState{
    max_position_size: u64,
    current_exposure: f64,
    orders_checked:u64,
    orders_rejected:u64,
    executor:Option<ActorRef<executor::ExecutorMessage>>
}

#[ractor::async_trait]
impl Actor for RiskManager {
    type Msg=RiskMessage;
    type State =RiskState;
    type Arguments = (u64,Option<ActorRef<ExecutorMessage>>);

    async fn pre_start(&self, myself: ActorRef<Self::Msg>, (max_position_size,executor): Self::Arguments) -> Result<Self::State, ActorProcessingErr> {
        println!("[RiskManager] starting with max position size: {}", max_position_size);
        Ok(RiskState{
            max_position_size,
            current_exposure:0.0,
            orders_checked:0,
            orders_rejected:0,
            executor,
        })
    }

    async fn handle(&self, myself: ActorRef<Self::Msg>, message: Self::Msg, state: &mut Self::State) -> Result<(), ActorProcessingErr> {
        match message { 
            RiskMessage::CheckOrder(order) => {
                state.orders_checked += 1;
                if order.quantity>state.max_position_size{
                    state.orders_rejected+=1;
                    println!("[RiskManager] rejected order {}:quantity {} exceeds limit {}",order.order_id,order.quantity,state.max_position_size);
                    return Ok(())
                }
                if order.order_id=="ORDER-FAIL"{
                    return Err(ActorProcessingErr::from("Risk calculation error: invalid market state"));
                }
                let order_value=order.quantity as f64 * order.price;
                state.current_exposure+=order_value;
                println!("[RiskManager] approved oder {}: exposure now {:.2}",order.order_id,state.current_exposure);
                if let Some(ref executor)=state.executor{
                    executor.cast(ExecutorMessage::ExecuteOrder(order))?;
                }
            }
            RiskMessage::GetExposure(reply) => {
                reply.send(state.current_exposure)?;
            }
            RiskMessage::Reset => {
                println!("[RiskManager] reset order");
                state.orders_rejected=0;
                state.orders_checked=0;
                state.current_exposure=0.0;
            }
        }
        Ok(())
    }
}