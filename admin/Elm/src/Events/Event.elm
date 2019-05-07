module Events.Event exposing
    ( Event
    , FullEvent
    , fullEventDecoder
    )

import Events.Occurrence exposing (Occurrence, occurrenceDecoder)
import Json.Decode as Decode
import Json.Encode as Encode


type alias Event =
    { name : String
    , teaser : String
    , description : String
    }


eventDecoder : Decode.Decoder Event
eventDecoder =
    Decode.map3
        (\name_ teaser_ description_ ->
            { name = name_
            , teaser = teaser_
            , description = description_
            }
        )
        (Decode.field "name" Decode.string)
        (Decode.field "teaser" Decode.string)
        (Decode.field "description" Decode.string)


type alias FullEvent =
    { event : Event
    , occurrences : List Occurrence
    }


fullEventDecoder : Decode.Decoder FullEvent
fullEventDecoder =
    Decode.map2
        (\event_ occurrences_ ->
            { event = event_
            , occurrences = occurrences_
            }
        )
        eventDecoder
        (Decode.field "occurrences" (Decode.list occurrenceDecoder))
