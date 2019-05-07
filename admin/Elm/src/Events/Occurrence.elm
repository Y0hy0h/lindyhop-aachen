module Events.Occurrence exposing
    ( Occurrence
    , OccurrenceTime
    , occurrenceDecoder
    , splitOccurrence
    )

import Date exposing (Date)
import Events.Location as Location exposing (Location, locationDecoder)
import Json.Decode as Decode
import Json.Encode as Encode
import Time
import Utils.SimpleTime as SimpleTime exposing (SimpleTime)


type alias Occurrence =
    { start : Time.Posix
    , duration : Duration
    , location : Location
    }


occurrenceDecoder : Decode.Decoder Occurrence
occurrenceDecoder =
    Decode.map3
        (\start_ duration_ location_ ->
            { start = start_
            , duration = duration_
            , location = location_
            }
        )
        (Decode.field "start" posixDecoder)
        (Decode.field "duration" durationDecoder)
        (Decode.field "location" locationDecoder)


posixDecoder : Decode.Decoder Time.Posix
posixDecoder =
    Decode.int
        -- Time.millisToPosix expects milliseconds
        |> Decode.map (\seconds -> seconds * 1000)
        |> Decode.map Time.millisToPosix


type Duration
    = Seconds Int


durationDecoder : Decode.Decoder Duration
durationDecoder =
    Decode.map
        Seconds
        Decode.int


type alias OccurrenceTime =
    { startTime : SimpleTime
    , duration : Duration
    , location : Location
    }


splitOccurrence : Time.Zone -> Occurrence -> ( Date, OccurrenceTime )
splitOccurrence zone occurrence =
    let
        startTime =
            SimpleTime.fromPosix zone occurrence.start

        occurrenceTime =
            { startTime = startTime
            , duration = occurrence.duration
            , location = occurrence.location
            }
    in
    ( Date.fromPosix zone occurrence.start
    , occurrenceTime
    )
