exception UndefinedEvent(string)
exception UndefinedContract(Ethers.ethAddress, int)

let getContractNameFromAddress = (contractAddress: Ethers.ethAddress, chainId: int): string => {
  switch (contractAddress->Ethers.ethAddressToString, chainId->Belt.Int.toString) {
  // TODO: make 'contracts' be per contract type/name, and have addresses as an array inside each contract.
  | ("0x2E645469f354BB4F5c8a05B3b30A929361cf77eC", "137") => "Gravatar"
  | _ => UndefinedContract(contractAddress, chainId)->raise
  }
}
let eventStringToEvent = (eventName: string, contractName: string): Types.eventName => {
  switch (eventName, contractName) {
  | ("TestEvent", "Gravatar") => GravatarContract_TestEventEvent
  | ("NewGravatar", "Gravatar") => GravatarContract_NewGravatarEvent
  | ("UpdatedGravatar", "Gravatar") => GravatarContract_UpdatedGravatarEvent
  | _ => UndefinedEvent(eventName)->raise
  }
}

module Gravatar = {
  let convertTestEventLogDescription = (log: Ethers.logDescription<'a>): Ethers.logDescription<
    Types.GravatarContract.TestEventEvent.eventArgs,
  > => {
    log->Obj.magic
  }

  let convertTestEventLog = async (
    logDescription: Ethers.logDescription<Types.GravatarContract.TestEventEvent.eventArgs>,
    ~log: Ethers.log,
    ~blockPromise: promise<Ethers.JsonRpcProvider.block>,
  ) => {
    let params: Types.GravatarContract.TestEventEvent.eventArgs = {
      id: logDescription.args.id,
      user: logDescription.args.user,
      contactDetails: logDescription.args.contactDetails,
    }
    let block = await blockPromise

    let testEventLog: Types.eventLog<Types.GravatarContract.TestEventEvent.eventArgs> = {
      params,
      blockNumber: block.number,
      blockTimestamp: block.timestamp,
      blockHash: log.blockHash,
      srcAddress: log.address->Ethers.ethAddressToString,
      transactionHash: log.transactionHash,
      transactionIndex: log.transactionIndex,
      logIndex: log.logIndex,
    }
    Types.GravatarContract_TestEvent(testEventLog)
  }

  let convertNewGravatarLogDescription = (log: Ethers.logDescription<'a>): Ethers.logDescription<
    Types.GravatarContract.NewGravatarEvent.eventArgs,
  > => {
    log->Obj.magic
  }

  let convertNewGravatarLog = async (
    logDescription: Ethers.logDescription<Types.GravatarContract.NewGravatarEvent.eventArgs>,
    ~log: Ethers.log,
    ~blockPromise: promise<Ethers.JsonRpcProvider.block>,
  ) => {
    let params: Types.GravatarContract.NewGravatarEvent.eventArgs = {
      id: logDescription.args.id,
      owner: logDescription.args.owner,
      displayName: logDescription.args.displayName,
      imageUrl: logDescription.args.imageUrl,
    }
    let block = await blockPromise

    let newGravatarLog: Types.eventLog<Types.GravatarContract.NewGravatarEvent.eventArgs> = {
      params,
      blockNumber: block.number,
      blockTimestamp: block.timestamp,
      blockHash: log.blockHash,
      srcAddress: log.address->Ethers.ethAddressToString,
      transactionHash: log.transactionHash,
      transactionIndex: log.transactionIndex,
      logIndex: log.logIndex,
    }
    Types.GravatarContract_NewGravatar(newGravatarLog)
  }

  let convertUpdatedGravatarLogDescription = (
    log: Ethers.logDescription<'a>,
  ): Ethers.logDescription<Types.GravatarContract.UpdatedGravatarEvent.eventArgs> => {
    log->Obj.magic
  }

  let convertUpdatedGravatarLog = async (
    logDescription: Ethers.logDescription<Types.GravatarContract.UpdatedGravatarEvent.eventArgs>,
    ~log: Ethers.log,
    ~blockPromise: promise<Ethers.JsonRpcProvider.block>,
  ) => {
    let params: Types.GravatarContract.UpdatedGravatarEvent.eventArgs = {
      id: logDescription.args.id,
      owner: logDescription.args.owner,
      displayName: logDescription.args.displayName,
      imageUrl: logDescription.args.imageUrl,
    }
    let block = await blockPromise

    let updatedGravatarLog: Types.eventLog<
      Types.GravatarContract.UpdatedGravatarEvent.eventArgs,
    > = {
      params,
      blockNumber: block.number,
      blockTimestamp: block.timestamp,
      blockHash: log.blockHash,
      srcAddress: log.address->Ethers.ethAddressToString,
      transactionHash: log.transactionHash,
      transactionIndex: log.transactionIndex,
      logIndex: log.logIndex,
    }
    Types.GravatarContract_UpdatedGravatar(updatedGravatarLog)
  }
}
