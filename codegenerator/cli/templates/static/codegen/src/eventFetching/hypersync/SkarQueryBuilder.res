module LogsQuery: HyperSyncTypes.LogsQuery = {
  type addressWithTopics = {
    address: Ethers.ethAddress,
    topics: array<array<Ethers.EventFilter.topic>>,
  }

  let makeRequestBody = (
    ~fromBlock,
    ~toBlockInclusive,
    ~addressesWithTopics: array<addressWithTopics>,
  ): Skar.QueryTypes.postQueryBody => {
    fromBlock,
    toBlockExclusive: toBlockInclusive + 1,
    logs: addressesWithTopics->Belt.Array.map(({address, topics}): Skar.QueryTypes.logParams => {
      {
        address: [address],
        topics,
      }
    }),
    fieldSelection: {
      log: [
        Address,
        BlockHash,
        BlockNumber,
        Data,
        LogIndex,
        TransactionHash,
        TransactionIndex,
        Topic0,
        Topic1,
        Topic2,
        Topic3,
        Removed,
      ],
      block: [Number, Timestamp],
    },
  }

  //Note this function can throw an error
  let checkFields = (event: SkarClient.ResponseTypes.event): HyperSyncTypes.logsQueryPageItem => {
    let log = event.log

    let blockTimestamp = event.block->Belt.Option.flatMap(b => b.timestamp)

    switch (
      blockTimestamp,
      log.address,
      log.blockHash,
      log.blockNumber,
      log.data,
      log.index,
      log.transactionHash,
      log.transactionIndex,
      log.removed,
      log.topics,
    ) {
    | (
        Some(blockTimestamp),
        Some(address),
        Some(blockHash),
        Some(blockNumber),
        Some(data),
        Some(index),
        Some(transactionHash),
        Some(transactionIndex),
        Some(removed),
        Some(topics),
      ) =>
      let topics = topics->Belt.Array.keepMap(Js.Nullable.toOption)

      let log: Ethers.log = {
        data,
        blockNumber,
        blockHash,
        address: Ethers.getAddressFromStringUnsafe(address),
        transactionHash,
        transactionIndex,
        logIndex: index,
        topics,
        removed,
      }

      let pageItem: HyperSyncTypes.logsQueryPageItem = {log, blockTimestamp}
      pageItem
    | _ =>
      let missingParams =
        [
          blockTimestamp->Utils.optionMapNone("log.timestamp"),
          log.address->Utils.optionMapNone("log.address"),
          log.blockHash->Utils.optionMapNone("log.blockHash-"),
          log.blockNumber->Utils.optionMapNone("log.blockNumber"),
          log.data->Utils.optionMapNone("log.data"),
          log.index->Utils.optionMapNone("log.index"),
          log.transactionHash->Utils.optionMapNone("log.transactionHash"),
          log.transactionIndex->Utils.optionMapNone("log.transactionIndex"),
          log.removed->Utils.optionMapNone("log.removed"),
        ]->Belt.Array.keepMap(v => v)

      HyperSyncTypes.UnexpectedMissingParamsExn({
        queryName: "queryLogsPage Skar",
        missingParams,
      })->raise
    }
  }

  let convertResponse = (res: SkarClient.ResponseTypes.response): HyperSyncTypes.queryResponse<
    HyperSyncTypes.logsQueryPage,
  > => {
    try {
      let {nextBlock, archiveHeight} = res
      let items = res.events->Belt.Array.map(event => event->checkFields)
      let page: HyperSyncTypes.logsQueryPage = {
        items,
        nextBlock,
        archiveHeight,
      }

      Ok(page)
    } catch {
    | HyperSyncTypes.UnexpectedMissingParamsExn(err) =>
      Error(HyperSyncTypes.UnexpectedMissingParams(err))
    }
  }

  let queryLogsPage = async (
    ~serverUrl,
    ~fromBlock,
    ~toBlock,
    ~addresses: array<Ethers.ethAddress>,
    ~topics,
  ): HyperSyncTypes.queryResponse<HyperSyncTypes.logsQueryPage> => {
    //TODO: This needs to be modified so that only related topics to addresses get passed in
    let addressesWithTopics = addresses->Belt.Array.flatMap(address => {
      [{address, topics}]
      // let address = address->Ethers.ethAddressToStringLower->Obj.magic
      // topics->Belt.Array.flatMap(topicsInner =>
      //   topicsInner->Belt.Array.map(topic => {address, topics: []})
      // )
    })
    let body = makeRequestBody(~fromBlock, ~toBlockInclusive=toBlock, ~addressesWithTopics)
    let skarClient = SkarClient.make({url: serverUrl})

    let res = await skarClient->SkarClient.sendReq(body)

    //Use ethArchive converter since the response is currently
    //Using the same layout
    res->convertResponse
  }
}

module BlockTimestampQuery: HyperSyncTypes.BlockTimestampQuery = {
  let makeRequestBody = (~fromBlock, ~toBlockInclusive): Skar.QueryTypes.postQueryBody => {
    fromBlock,
    toBlockExclusive: toBlockInclusive + 1,
    transactions: [{}],
    fieldSelection: {
      block: [Timestamp, Number],
    },
  }

  let convertResponse = (
    res: result<Skar.ResponseTypes.queryResponse, QueryHelpers.queryError>,
  ): HyperSyncTypes.queryResponse<HyperSyncTypes.blockTimestampPage> => {
    switch res {
    | Error(e) => Error(HyperSyncTypes.QueryError(e))
    | Ok(successRes) =>
      let {nextBlock, archiveHeight, data} = successRes

      data
      ->Belt.Array.flatMap(item => {
        item.blocks->Belt.Option.mapWithDefault([], blocks => {
          blocks->Belt.Array.map(
            block => {
              switch (block.number, block.timestamp) {
              | (Some(blockNumber), Some(blockTimestamp)) =>
                let timestamp = blockTimestamp->Ethers.BigInt.toInt->Belt.Option.getExn
                Ok(
                  (
                    {
                      timestamp,
                      blockNumber,
                    }: HyperSyncTypes.blockNumberAndTimestamp
                  ),
                )
              | _ =>
                let missingParams =
                  [
                    block.number->Utils.optionMapNone("block.number"),
                    block.timestamp->Utils.optionMapNone("block.timestamp"),
                  ]->Belt.Array.keepMap(p => p)

                Error(
                  HyperSyncTypes.UnexpectedMissingParams({
                    queryName: "queryBlockTimestampsPage Skar",
                    missingParams,
                  }),
                )
              }
            },
          )
        })
      })
      ->Utils.mapArrayOfResults
      ->Belt.Result.map((items): HyperSyncTypes.blockTimestampPage => {
        nextBlock,
        archiveHeight,
        items,
      })
    }
  }

  let queryBlockTimestampsPage = async (
    ~serverUrl,
    ~fromBlock,
    ~toBlock,
  ): HyperSyncTypes.queryResponse<HyperSyncTypes.blockTimestampPage> => {
    let body = makeRequestBody(~fromBlock, ~toBlockInclusive=toBlock)

    let res = await Skar.executeSkarQuery(~postQueryBody=body, ~serverUrl)

    //Use ethArchive converter since the response is currently
    //Using the same layout
    res->convertResponse
  }
}

module HeightQuery: HyperSyncTypes.HeightQuery = {
  let getHeightWithRetry = EthArchiveQueryBuilder.HeightQuery.getHeightWithRetry

  //Poll for a height greater than the given blocknumber.
  //Used for waiting until there is a new block to index
  let pollForHeightGtOrEq = EthArchiveQueryBuilder.HeightQuery.pollForHeightGtOrEq
}
