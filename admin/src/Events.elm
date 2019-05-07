module Events exposing
    ( Event, Occurrence, Location, Store, Events, Locations
    , fetchEvents, createEvent, readEvent, updateEvent, deleteEvent, createLocation, readLocation, updateLocation, deleteLocation
    , locations, events, mapEvents, mapLocations
    )

{-| Fetches, stores, and makes accessible the events from the backend.


# Types

@docs Event, Occurrence, Location, Store, Events, Locations


# API

@docs fetchEvents, createEvent, readEvent, updateEvent, deleteEvent, createLocation, readLocation, updateLocation, deleteLocation


# Access

@docs locations, events, mapEvents, mapLocations

-}

import Dict exposing (Dict)
import Dict.Extra as Dict
import Http
import IdDict exposing (Id, IdDict, UnsafeId, decodeUnsafeId)
import Json.Decode as Decode
import Json.Encode as Encode
import List.Extra as List
import Url.Builder
import Utils.NaiveDateTime as Naive exposing (DateTime, Duration)


type alias Event =
    { name : String
    , teaser : String
    , description : String
    , occurrences : List Occurrence
    }


type alias Occurrence =
    { start : DateTime
    , duration : Duration
    , locationId : Id Location
    }


type alias Location =
    { name : String
    , address : String
    }



-- Store


{-| Event container.
-}
type Store
    = Store Locations Events


type alias Locations =
    IdDict Location


type alias Events =
    IdDict Event


{-| Extracts the locations.
-}
locations : Store -> IdDict Location
locations (Store locs _) =
    locs


events : Store -> IdDict Event
events (Store _ evts) =
    evts


mapLocations : (Id Location -> Location -> b) -> Store -> List b
mapLocations mapping store =
    locations store
        |> IdDict.map mapping


mapEvents : (Id Event -> Event -> b) -> Store -> List b
mapEvents mapping store =
    events store
        |> IdDict.map mapping



-- Init


apiUrl : List String -> String
apiUrl path =
    let
        base =
            [ "api" ]
    in
    Url.Builder.absolute (base ++ path) []


type HttpMethod
    = Get
    | Post
    | Put
    | Delete


stringFromHttpMethod : HttpMethod -> String
stringFromHttpMethod method =
    case method of
        Get ->
            "GET"

        Post ->
            "POST"

        Put ->
            "PUT"

        Delete ->
            "DELETE"


apiRequest :
    { method : HttpMethod
    , url : List String
    , body : Maybe Encode.Value
    , expect : Http.Expect msg
    }
    -> Cmd msg
apiRequest { method, url, body, expect } =
    Http.request
        { method = stringFromHttpMethod method
        , headers = []
        , url = apiUrl url
        , body =
            case body of
                Just json ->
                    Http.jsonBody json

                Nothing ->
                    Http.emptyBody
        , expect = expect
        , timeout = Nothing
        , tracker = Nothing
        }


{-| The HTTP command to fetch the events.
-}
fetchEvents : (Result Http.Error Store -> msg) -> Cmd msg
fetchEvents toMsg =
    apiRequest
        { method = Get
        , url = [ "events" ]
        , body = Nothing
        , expect = Http.expectJson toMsg decodeEvents
        }


createEvent : Event -> (Result Http.Error UnsafeId -> msg) -> Cmd msg
createEvent event toMsg =
    apiRequest
        { method = Post
        , url = [ "events" ]
        , body = Just <| encodeEvent event
        , expect = Http.expectJson toMsg decodeUnsafeId
        }


readEvent : Locations -> Id Event -> (Result Http.Error Event -> msg) -> Cmd msg
readEvent locs id toMsg =
    apiRequest
        { method = Get
        , url = [ "events", IdDict.encodeIdForUrl id ]
        , body = Nothing
        , expect = Http.expectJson toMsg (decodeEvent locs)
        }


updateEvent : Id Event -> Event -> (Result Http.Error () -> msg) -> Cmd msg
updateEvent id event toMsg =
    apiRequest
        { method = Put
        , url = [ "events", IdDict.encodeIdForUrl id ]
        , body = Just <| encodeEvent event
        , expect = Http.expectWhatever toMsg
        }


deleteEvent : Locations -> Id Event -> (Result Http.Error Event -> msg) -> Cmd msg
deleteEvent locs id toMsg =
    apiRequest
        { method = Delete
        , url = [ "events", IdDict.encodeIdForUrl id ]
        , body = Nothing
        , expect = Http.expectJson toMsg (decodeEvent locs)
        }


createLocation : Location -> (Result Http.Error UnsafeId -> msg) -> Cmd msg
createLocation location toMsg =
    apiRequest
        { method = Post
        , url = [ "locations" ]
        , body = Just <| encodeLocation location
        , expect = Http.expectJson toMsg decodeUnsafeId
        }


readLocation : Id Location -> (Result Http.Error Location -> msg) -> Cmd msg
readLocation id toMsg =
    apiRequest
        { method = Get
        , url = [ "locations", IdDict.encodeIdForUrl id ]
        , body = Nothing
        , expect = Http.expectJson toMsg decodeLocation
        }


updateLocation : Id Location -> Location -> (Result Http.Error () -> msg) -> Cmd msg
updateLocation id location toMsg =
    apiRequest
        { method = Put
        , url = [ "locations", IdDict.encodeIdForUrl id ]
        , body = Just <| encodeLocation location
        , expect = Http.expectWhatever toMsg
        }


deleteLocation : Id Location -> (Result Http.Error Location -> msg) -> Cmd msg
deleteLocation id toMsg =
    apiRequest
        { method = Delete
        , url = [ "locations", IdDict.encodeIdForUrl id ]
        , body = Nothing
        , expect = Http.expectJson toMsg decodeLocation
        }



-- Decode


decodeEvents : Decode.Decoder Store
decodeEvents =
    let
        defaultLocation =
            Location "" ""

        defaultEvent =
            Event "" "" "" []
    in
    Decode.field "locations" (IdDict.decodeIdDict defaultLocation decodeLocation)
        |> Decode.andThen
            (\locs ->
                Decode.map
                    (Store locs)
                    (Decode.field "events" (IdDict.decodeIdDict defaultEvent (decodeEvent locs)))
            )


decodeEvent : IdDict Location -> Decode.Decoder Event
decodeEvent locs =
    Decode.map4
        Event
        (Decode.field "name" Decode.string)
        (Decode.field "teaser" Decode.string)
        (Decode.field "description" Decode.string)
        (Decode.field "occurrences" (Decode.list (decodeOccurrence locs)))


decodeOccurrence : IdDict Location -> Decode.Decoder Occurrence
decodeOccurrence locs =
    Decode.field "location_id" decodeUnsafeId
        |> Decode.andThen
            (\unsafeId ->
                case IdDict.validate unsafeId locs of
                    Just id ->
                        Decode.succeed id

                    Nothing ->
                        Decode.fail "Invalid location id."
            )
        |> Decode.map3
            Occurrence
            (Decode.field "start" Naive.decodeDateTime)
            (Decode.field "duration" Naive.decodeMinutes)


decodeLocation : Decode.Decoder Location
decodeLocation =
    Decode.map2
        Location
        (Decode.field "name" Decode.string)
        (Decode.field "address" Decode.string)



-- Encode


encodeEvent : Event -> Encode.Value
encodeEvent event =
    Encode.object
        [ ( "name", Encode.string event.name )
        , ( "teaser", Encode.string event.teaser )
        , ( "description", Encode.string event.description )
        , ( "occurrences", Encode.list encodeOccurrence event.occurrences )
        ]


encodeOccurrence : Occurrence -> Encode.Value
encodeOccurrence occurrence =
    Encode.object
        [ ( "start", Naive.encodeDateTime occurrence.start )
        , ( "duration", Naive.encodeAsMinutes occurrence.duration )
        , ( "location_id", IdDict.encodeId occurrence.locationId )
        ]


encodeLocation : Location -> Encode.Value
encodeLocation location =
    Encode.object
        [ ( "name", Encode.string location.name )
        , ( "address", Encode.string location.address )
        ]
