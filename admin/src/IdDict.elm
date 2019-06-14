module IdDict exposing (Id, IdDict, UnsafeId, decodeIdDict, decodeUnsafeId, encodeId, encodeIdForUrl, from, get, map, validate)

import Dict exposing (Dict)
import Json.Decode as Decode
import Json.Encode as Encode


type IdDict a
    = IdDict a (Dict String a)


from : a -> Dict String a -> IdDict a
from default raw =
    IdDict default raw


get : Id a -> IdDict a -> a
get (Id rawId) (IdDict default dict) =
    Dict.get rawId dict
        |> Maybe.withDefault default


validate : UnsafeId -> IdDict a -> Maybe (Id a)
validate rawId (IdDict _ dict) =
    if Dict.member rawId dict then
        Just (Id rawId)

    else
        Nothing


map : (Id a -> a -> b) -> IdDict a -> List b
map mapping (IdDict _ dict) =
    Dict.toList dict
        |> List.map (\( k, v ) -> mapping (Id k) v)


decodeIdDict : a -> Decode.Decoder a -> Decode.Decoder (IdDict a)
decodeIdDict default decodeItem =
    Decode.dict decodeItem
        |> Decode.map (IdDict default)


{-| Wrapper for ids to prevent mixing of ids from different types at compile time.
-}
type Id a
    = Id String


type alias UnsafeId =
    String


decodeUnsafeId : Decode.Decoder UnsafeId
decodeUnsafeId =
    Decode.string


encodeIdForUrl : Id a -> String
encodeIdForUrl (Id raw) =
    raw


encodeId : Id a -> Encode.Value
encodeId (Id raw) =
    Encode.string raw
