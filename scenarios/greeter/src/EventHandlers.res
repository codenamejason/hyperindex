open Types

Handlers.GreeterContract.registerNewGreetingLoadEntities((~event, ~context) => {
  context.greeting.greetingWithChangesLoad(event.params.user->Ethers.ethAddressToString)
})

Handlers.GreeterContract.registerNewGreetingHandler((~event, ~context) => {
  let currentGreeterOpt = context.greeting.greetingWithChanges()

  switch currentGreeterOpt {
  | Some(existingGreeter) => {
      let greetingObject: greetingEntity = {
        id: event.params.user->Ethers.ethAddressToString,
        latestGreeting: event.params.greeting,
        numberOfGreetings: existingGreeter.numberOfGreetings + 1,
      }

      context.greeting.update(greetingObject)
    }

  | None =>
    let greetingObject: greetingEntity = {
      id: event.params.user->Ethers.ethAddressToString,
      latestGreeting: event.params.greeting,
      numberOfGreetings: 1,
    }

    context.greeting.insert(greetingObject)
  }
})

Handlers.GreeterContract.registerClearGreetingLoadEntities((~event, ~context) => {
  context.greeting.greetingWithChangesLoad(event.params.user->Ethers.ethAddressToString)
  ()
})

Handlers.GreeterContract.registerClearGreetingHandler((~event, ~context) => {
  
   let currentGreeterOpt = context.greeting.greetingWithChanges()

  if currentGreeterOpt->Belt.Option.isSome {
    
    let greetingObject: greetingEntity = {
          id: event.params.user->Ethers.ethAddressToString,
          latestGreeting: "",
          numberOfGreetings: currentGreeterOpt->Belt.Option.mapWithDefault(1, greeting =>
       greeting.numberOfGreetings
     )   
        }

  context.greeting.update(greetingObject)
}
})
