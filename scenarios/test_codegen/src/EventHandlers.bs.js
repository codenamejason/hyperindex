// Generated by ReScript, PLEASE EDIT WITH CARE
'use strict';

var Curry = require("rescript/lib/js/curry.js");
var Ethers = require("generated/src/bindings/Ethers.bs.js");
var Handlers = require("generated/src/Handlers.bs.js");
var Belt_Option = require("rescript/lib/js/belt_Option.js");
var Caml_option = require("rescript/lib/js/caml_option.js");

Handlers.GravatarContract.registerNewGravatarLoadEntities(function (param, param$1) {
      
    });

Handlers.GravatarContract.registerNewGravatarHandler(function ($$event, context) {
      Curry._1(context.gravatar.insert, {
            id: $$event.params.id.toString(),
            owner: Ethers.ethAddressToString($$event.params.owner),
            displayName: $$event.params.displayName,
            imageUrl: $$event.params.imageUrl,
            updatesCount: 1,
            bigIntTest: BigInt(1),
            bigIntOption: Caml_option.some(BigInt(1))
          });
    });

Handlers.GravatarContract.registerUpdatedGravatarLoadEntities(function ($$event, context) {
      Curry._1(context.gravatar.gravatarWithChangesLoad, $$event.params.id.toString());
    });

Handlers.GravatarContract.registerUpdatedGravatarHandler(function ($$event, context) {
      var updatesCount = Belt_Option.mapWithDefault(Curry._1(context.gravatar.gravatarWithChanges, undefined), 1, (function (gravatar) {
              return gravatar.updatesCount + 1 | 0;
            }));
      Curry._1(context.gravatar.update, {
            id: $$event.params.id.toString(),
            owner: Ethers.ethAddressToString($$event.params.owner),
            displayName: $$event.params.displayName,
            imageUrl: $$event.params.imageUrl,
            updatesCount: updatesCount,
            bigIntTest: BigInt(1),
            bigIntOption: Caml_option.some(BigInt(1))
          });
    });

/*  Not a pure module */
