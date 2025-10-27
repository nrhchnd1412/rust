use ractor::{ActorRef, Actor, ActorProcessingErr, SupervisionEvent, RpcReplyPort};
use std::time::Duration;
use tokio::time::sleep;
use crate::order_book;
use crate::order_book::OrderSide;

pub enum ExecutorMessage{
    ExecuteOrder(order_book::Order),
    GetExecutedCount(ractor::RpcReplyPort<u64>),
}

pub struct OrderExecutor;

pub struct ExecutorState {
    executed_count:u64,
}

#[ractor::async_trait]
impl Actor for OrderExecutor {
    type Msg=ExecutorMessage;
    type State=ExecutorState;
    type Arguments = ();

    async fn pre_start(&self, myself: ActorRef<Self::Msg>, args: Self::Arguments) -> Result<Self::State, ActorProcessingErr> {
        println!("[Executor] starting oder executor");
        Ok(ExecutorState{
            executed_count:0,
        })
    }
    async fn handle(&self, myself: ActorRef<Self::Msg>, message: Self::Msg, state: &mut Self::State) -> Result<(), ActorProcessingErr> {
        match message {
            ExecutorMessage::ExecuteOrder(order) => {
                println!("[Executor] executing order {}:{} {} @ {:.2}",
                order.order_id,
                match order.side{
                    OrderSide::Buy => "BUY",
                    OrderSide::Sell => "SELL",
                }, order.quantity,order.price,
                );
                sleep(Duration::from_millis(100)).await;
                state.executed_count += 1;
                println!("[Executor] order {} filled",order.order_id);
            }
            ExecutorMessage::GetExecutedCount(reply) => {
                reply.send(state.executed_count)?;
            }
        }
        Ok(())
    }
}