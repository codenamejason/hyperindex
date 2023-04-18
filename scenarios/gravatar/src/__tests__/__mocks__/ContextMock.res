open Jest
open Types
let insertMock = (id => id)->JestJs.fn
// let getByIdMock = (id => id)->JestJs.fn
let updateMock = (id => id)->JestJs.fn
// let loadedEntitiesMock = {
//       /// TODO: add named entities (this is hardcoded)
//       gravatarWithChanges: () => MockEntities.gravatarEntity1,
//       insert: gravatarEntity => (),
//       update: gravatarEntity => (),
//       delete: id => (),
//     }

let mockNewGravatarContext: Types.GravatarContract.NewGravatarEvent.context = {
  gravatar: {
    gravatarWithChanges: () => None, // TODO remove this once codegen is fixed!
    insert: gravatarInsert => {
      insertMock->MockJs.fn(gravatarInsert.id)->ignore
    },
    update: gravatarUpdate => {
      updateMock->MockJs.fn(gravatarUpdate.id)->ignore
    },
    delete: id => (),
  },
}

let mockUpdateGravatarContext: Types.GravatarContract.UpdatedGravatarEvent.context = {
  gravatar: {
    gravatarWithChanges: () => Some(MockEntities.gravatarEntity1),
    insert: gravatarInsert => {
      insertMock->MockJs.fn(gravatarInsert.id)->ignore
    },
    update: gravatarUpdate => {
      updateMock->MockJs.fn(gravatarUpdate.id)->ignore
    },
    delete: id => (),
  },
}
