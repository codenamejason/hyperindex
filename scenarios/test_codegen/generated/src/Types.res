//*************
//***ENTITIES**
//*************

@genType.as("Id")
type id = string

//nested subrecord types

@spice
type contactDetails = {
  name: string,
  email: string,
}

type entityRead =
  | UserRead(id)
  | GravatarRead(id)

let entitySerialize = (entity: entityRead) => {
  switch entity {
  | UserRead(id) => `user${id}`
  | GravatarRead(id) => `gravatar${id}`
  }
}

type rawEventsEntity = {
  @as("chain_id") chainId: int,
  @as("event_id") eventId: Ethers.BigInt.t,
  @as("block_number") blockNumber: int,
  @as("log_index") logIndex: int,
  @as("transaction_index") transactionIndex: int,
  @as("transaction_hash") transactionHash: string,
  @as("src_address") srcAddress: string,
  @as("block_hash") blockHash: string,
  @as("block_timestamp") blockTimestamp: int,
  @as("event_type") eventType: Js.Json.t,
  params: Js.Json.t,
}

@genType
type userEntity = {
  id: string,
  address: string,
  gravatar: option<id>,
}

type userEntitySerialized = {
  id: string,
  address: string,
  gravatar: option<id>,
}

let serializeUserEntity = (entity: userEntity): userEntitySerialized => {
  {
    id: entity.id,
    address: entity.address,
    gravatar: entity.gravatar,
  }
}

let deserializeUserEntity = (entitySerialized: userEntitySerialized): userEntity => {
  {
    id: entitySerialized.id,
    address: entitySerialized.address,
    gravatar: entitySerialized.gravatar,
  }
}

@genType
type gravatarEntity = {
  id: string,
  owner: id,
  displayName: string,
  imageUrl: string,
  updatesCount: int,
  bigIntTest: Ethers.BigInt.t,
  bigIntOption: option<Ethers.BigInt.t>,
}

type gravatarEntitySerialized = {
  id: string,
  owner: id,
  displayName: string,
  imageUrl: string,
  updatesCount: int,
  bigIntTest: string,
  bigIntOption: option<string>,
}

let serializeGravatarEntity = (entity: gravatarEntity): gravatarEntitySerialized => {
  {
    id: entity.id,
    owner: entity.owner,
    displayName: entity.displayName,
    imageUrl: entity.imageUrl,
    updatesCount: entity.updatesCount,
    bigIntTest: entity.bigIntTest->Ethers.BigInt.toString,
    bigIntOption: entity.bigIntOption->Belt.Option.map(opt => opt->Ethers.BigInt.toString),
  }
}

let deserializeGravatarEntity = (entitySerialized: gravatarEntitySerialized): gravatarEntity => {
  {
    id: entitySerialized.id,
    owner: entitySerialized.owner,
    displayName: entitySerialized.displayName,
    imageUrl: entitySerialized.imageUrl,
    updatesCount: entitySerialized.updatesCount,
    bigIntTest: entitySerialized.bigIntTest->Ethers.BigInt.fromStringUnsafe,
    bigIntOption: entitySerialized.bigIntOption->Belt.Option.map(opt =>
      opt->Ethers.BigInt.fromStringUnsafe
    ),
  }
}

type entity =
  | UserEntity(userEntity)
  | GravatarEntity(gravatarEntity)

type crud = Create | Read | Update | Delete

type inMemoryStoreRow<'a> = {
  crud: crud,
  entity: 'a,
}

//*************
//**CONTRACTS**
//*************

@genType
type eventLog<'a> = {
  params: 'a,
  blockNumber: int,
  blockTimestamp: int,
  blockHash: string,
  srcAddress: string,
  transactionHash: string,
  transactionIndex: int,
  logIndex: int,
}

module GravatarContract = {
  module TestEventEvent = {
    @spice @genType
    type eventArgs = {
      id: Ethers.BigInt.t,
      user: Ethers.ethAddress,
      contactDetails: contactDetails,
    }
    type userEntityHandlerContext = {
      insert: userEntity => unit,
      update: userEntity => unit,
      delete: id => unit,
    }
    type gravatarEntityHandlerContext = {
      insert: gravatarEntity => unit,
      update: gravatarEntity => unit,
      delete: id => unit,
    }
    @genType
    type context = {
      user: userEntityHandlerContext,
      gravatar: gravatarEntityHandlerContext,
    }

    @genType
    type loaderContext = {}
  }
  module NewGravatarEvent = {
    @spice @genType
    type eventArgs = {
      id: Ethers.BigInt.t,
      owner: Ethers.ethAddress,
      displayName: string,
      imageUrl: string,
    }
    type userEntityHandlerContext = {
      insert: userEntity => unit,
      update: userEntity => unit,
      delete: id => unit,
    }
    type gravatarEntityHandlerContext = {
      insert: gravatarEntity => unit,
      update: gravatarEntity => unit,
      delete: id => unit,
    }
    @genType
    type context = {
      user: userEntityHandlerContext,
      gravatar: gravatarEntityHandlerContext,
    }

    @genType
    type loaderContext = {}
  }
  module UpdatedGravatarEvent = {
    @spice @genType
    type eventArgs = {
      id: Ethers.BigInt.t,
      owner: Ethers.ethAddress,
      displayName: string,
      imageUrl: string,
    }
    type userEntityHandlerContext = {
      insert: userEntity => unit,
      update: userEntity => unit,
      delete: id => unit,
    }
    type gravatarEntityHandlerContext = {
      gravatarWithChanges: unit => option<gravatarEntity>,
      insert: gravatarEntity => unit,
      update: gravatarEntity => unit,
      delete: id => unit,
    }
    @genType
    type context = {
      user: userEntityHandlerContext,
      gravatar: gravatarEntityHandlerContext,
    }

    type gravatarEntityLoaderContext = {gravatarWithChangesLoad: id => unit}

    @genType
    type loaderContext = {gravatar: gravatarEntityLoaderContext}
  }
}

type event =
  | GravatarContract_TestEvent(eventLog<GravatarContract.TestEventEvent.eventArgs>)
  | GravatarContract_NewGravatar(eventLog<GravatarContract.NewGravatarEvent.eventArgs>)
  | GravatarContract_UpdatedGravatar(eventLog<GravatarContract.UpdatedGravatarEvent.eventArgs>)

type eventAndContext =
  | GravatarContract_TestEventWithContext(
      eventLog<GravatarContract.TestEventEvent.eventArgs>,
      GravatarContract.TestEventEvent.context,
    )
  | GravatarContract_NewGravatarWithContext(
      eventLog<GravatarContract.NewGravatarEvent.eventArgs>,
      GravatarContract.NewGravatarEvent.context,
    )
  | GravatarContract_UpdatedGravatarWithContext(
      eventLog<GravatarContract.UpdatedGravatarEvent.eventArgs>,
      GravatarContract.UpdatedGravatarEvent.context,
    )

@spice
type eventName =
  | @spice.as("GravatarContract_TestEventEvent") GravatarContract_TestEventEvent
  | @spice.as("GravatarContract_NewGravatarEvent") GravatarContract_NewGravatarEvent
  | @spice.as("GravatarContract_UpdatedGravatarEvent") GravatarContract_UpdatedGravatarEvent

let eventNameToString = (eventName: eventName) =>
  switch eventName {
  | GravatarContract_TestEventEvent => "TestEvent"
  | GravatarContract_NewGravatarEvent => "NewGravatar"
  | GravatarContract_UpdatedGravatarEvent => "UpdatedGravatar"
  }
