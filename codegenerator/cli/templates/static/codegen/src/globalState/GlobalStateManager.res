open Belt
module type State = {
  type t
  type action
  type task

  let taskReducer: (t, task, ~dispatchAction: action => unit) => unit
  let actionReducer: (t, action) => (t, array<task>)
}

module MakeManager = (S: State) => {
  type t = {mutable state: S.t, stateUpdatedHook: option<S.t => unit>}

  let make = (~stateUpdatedHook: option<S.t => unit>=?, state: S.t) => {state, stateUpdatedHook}

  let rec dispatchAction = (self: t, action: S.action) => {
    try {
      let (nextState, nextTasks) = S.actionReducer(self.state, action)
      self.state = nextState
      switch self.stateUpdatedHook {
      | Some(hook) => hook(nextState)
      | None => ()
      }
      nextTasks->Array.forEach(task => dispatchTask(self, task))
    } catch {
    | e =>
      e->ErrorHandling.make(~msg="Indexer has failed with an unxpected error")->ErrorHandling.log
      NodeJsLocal.process->NodeJsLocal.exitWithCode(Failure)
    }
  }
  and dispatchTask = (self, task: S.task) => Js.Global.setTimeout(() => {
      S.taskReducer(self.state, task, ~dispatchAction=dispatchAction(self))
    }, 0)->ignore

  let getState = self => self.state
}

module Manager = MakeManager(GlobalState)
include Manager
