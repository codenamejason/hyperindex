//************
//** EVENTS **
//************

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

type newGravatarEvent = {
  id: int,
  owner: string,
  displayName: string,
  imageUrl: string,
}

type updatedGravatarEvent = {
  id: int,
  owner: string,
  displayName: string,
  imageUrl: string,
}

type event =
  | NewGravatar(eventLog<newGravatarEvent>)
  | UpdatedGravatar(eventLog<updatedGravatarEvent>)

//*************
//***ENTITIES**
//*************

type id = string

type entityRead = GravatarRead(id)

let entitySerialize = (entity: entityRead) => {
  switch entity {
  | GravatarRead(id) => `gravatar${id}`
  }
}

type gravatarEntity = {
  id: string,
  owner: string,
  displayName: string,
  imageUrl: string,
  updatesCount: int,
}

type entity = GravatarEntity(gravatarEntity)

type crud = Create | Read | Update | Delete

type inMemoryStoreRow<'a> = {
  crud: crud,
  entity: 'a,
}

//*************
//** CONTEXT **
//*************

type loadedEntitiesReader<'a> = {
  getById: id => option<'a>,
  getAllLoaded: unit => array<'a>,
}

type entityController<'a> = {
  insert: 'a => unit,
  update: 'a => unit,
  loadedEntities: loadedEntitiesReader<'a>,
}

type gravatarController = entityController<gravatarEntity>

type context = {@as("Gravatar") gravatar: gravatarController}
