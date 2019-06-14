module Routes exposing (Route(..), routeName, toRelativeUrl, toRoute)

import Url exposing (Url)
import Url.Parser as Parser exposing ((</>), Parser, map, s, string, top)


type Route
    = Overview
    | CreateEvent
    | EditEvent String
    | CreateLocation
    | EditLocation String
    | NotFound


toRoute : Url -> Route
toRoute url =
    Maybe.withDefault NotFound (Parser.parse routeParser url)


toRelativeUrl : Route -> String
toRelativeUrl route =
    let
        parts =
            case route of
                Overview ->
                    []

                CreateEvent ->
                    [ "event" ]

                EditEvent id ->
                    [ "event", id ]

                CreateLocation ->
                    [ "location" ]

                EditLocation id ->
                    [ "location", id ]

                NotFound ->
                    []
    in
    "/" ++ String.join "/" (root ++ parts)


routeName : Route -> String
routeName route =
    case route of
        Overview ->
            "Admin"

        CreateEvent ->
            "Veranstaltung"

        EditEvent id ->
            "Veranstaltung"

        CreateLocation ->
            "Ort"

        EditLocation id ->
            "Ort"

        NotFound ->
            "Nicht gefunden"


root : List String
root =
    [ "admin" ]


routeParser : Parser (Route -> a) a
routeParser =
    let
        rootUrl =
            List.map s root
                |> List.foldl (\next currentPath -> currentPath </> next) top
    in
    Parser.oneOf
        [ map Overview rootUrl
        , map CreateEvent (rootUrl </> s "event")
        , map EditEvent (rootUrl </> s "event" </> string)
        , map CreateLocation (rootUrl </> s "location")
        , map EditLocation (rootUrl </> s "location" </> string)
        ]
