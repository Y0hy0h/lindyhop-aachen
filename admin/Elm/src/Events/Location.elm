module Events.Location exposing
    ( Location
    , locationDecoder
    )

import Json.Decode as Decode
import Json.Encode as Encode


type alias Location =
    { name : String
    , address : String
    }


locationDecoder : Decode.Decoder Location
locationDecoder =
    Decode.map2
        (\name_ address_ ->
            { name = name_
            , address = address_
            }
        )
        (Decode.field "name" Decode.string)
        (Decode.field "address" Decode.string)
