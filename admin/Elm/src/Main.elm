module Main exposing (main)

import Browser
import Browser.Navigation as Browser
import Date exposing (Date)
import Events exposing (EventsTree, OccurrenceList)
import Events.Event as Event exposing (Event, FullEvent, fullEventDecoder)
import Events.Location as Location exposing (Location)
import Events.Occurrence as Occurrence exposing (Occurrence, OccurrenceTime)
import Html exposing (Html, div, h1, h2, li, ol, p, text)
import Http
import Json.Decode as Decode
import Task
import Time
import Url exposing (Url)
import Utils.SimpleTime as SimpleTime exposing (SimpleTime)



-- Main


main =
    Browser.application
        { init = init
        , view = view
        , update = appUpdate
        , subscriptions = subscriptions
        , onUrlRequest = \_ -> AppNoOp
        , onUrlChange = \_ -> AppNoOp
        }



-- Model


type AppModel
    = Loading
    | LoadedEvents (List FullEvent)
    | ErrorLoadingEvents Http.Error
    | LoadedTimezone Time.Zone
    | Loaded Model


type alias Model =
    { timezone : Time.Zone
    , events : OccurrenceList
    }



-- I/O


init : () -> Url -> Browser.Key -> ( AppModel, Cmd AppMsg )
init _ url key =
    let
        getTimezone =
            Task.perform FetchedTimezone Time.here

        getEvents =
            Http.get
                { url = "/api/events"
                , expect = Http.expectJson FetchedEvents (Decode.list fullEventDecoder)
                }
    in
    ( Loading, Cmd.batch [ getTimezone, getEvents ] )


subscriptions : AppModel -> Sub AppMsg
subscriptions model =
    Sub.none



-- Update


type AppMsg
    = AppNoOp
    | FetchedTimezone Time.Zone
    | FetchedEvents (Result Http.Error (List FullEvent))
    | Sub Msg


appUpdate : AppMsg -> AppModel -> ( AppModel, Cmd AppMsg )
appUpdate msg model =
    case msg of
        AppNoOp ->
            ( model, Cmd.none )

        FetchedTimezone zone ->
            let
                newModel =
                    load (Just zone) Nothing model
            in
            ( newModel, Cmd.none )

        FetchedEvents result ->
            let
                newModel =
                    case result of
                        Ok events ->
                            load Nothing (Just events) model

                        Err error ->
                            ErrorLoadingEvents error
            in
            ( newModel, Cmd.none )

        Sub subMsg ->
            case model of
                Loaded loadedModel ->
                    update subMsg loadedModel
                        |> Tuple.mapBoth Loaded (Cmd.map Sub)

                _ ->
                    ( model, Cmd.none )


load : Maybe Time.Zone -> Maybe (List FullEvent) -> AppModel -> AppModel
load maybeZone maybeEvents model =
    let
        loaded : Time.Zone -> List FullEvent -> AppModel
        loaded zone events =
            Loaded
                { timezone = zone
                , events =
                    Events.occurrenceListFromEventsTree events
                }
    in
    case model of
        Loading ->
            case ( maybeZone, maybeEvents ) of
                ( Just zone, Just events ) ->
                    loaded zone events

                ( Just zone, Nothing ) ->
                    LoadedTimezone zone

                ( Nothing, Just events ) ->
                    LoadedEvents events

                ( Nothing, Nothing ) ->
                    Loading

        LoadedEvents events ->
            case maybeZone of
                Just zone ->
                    loaded zone events

                Nothing ->
                    LoadedEvents events

        LoadedTimezone zone ->
            case maybeEvents of
                Just events ->
                    loaded zone events

                Nothing ->
                    LoadedTimezone zone

        ErrorLoadingEvents _ ->
            model

        Loaded _ ->
            model


type Msg
    = NoOp


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        NoOp ->
            ( model, Cmd.none )



-- View


view : AppModel -> Browser.Document AppMsg
view appModel =
    let
        html : List (Html AppMsg)
        html =
            case appModel of
                Loading ->
                    viewLoading

                LoadedEvents _ ->
                    viewLoading

                LoadedTimezone _ ->
                    viewLoading

                ErrorLoadingEvents error ->
                    viewError error

                Loaded model ->
                    viewLoaded model
                        |> List.map (Html.map Sub)
    in
    { title = "Lindy Hop Aachen Admin"
    , body = html
    }


viewLoading : List (Html AppMsg)
viewLoading =
    [ h1 [] [ text "Admin" ]
    , p [] [ text "Lädt..." ]
    ]


viewError : Http.Error -> List (Html AppMsg)
viewError error =
    [ h1 [] [ text "Admin" ]
    , p [] [ text "Beim Laden der Events ist ein Fehler passiert." ]
    ]


viewLoaded : Model -> List (Html Msg)
viewLoaded model =
    [ h1 [] [ text "Admin" ]
    , ol []
        (List.map
            (\( date, occurrences ) ->
                li []
                    [ text <| stringFromDate date
                    , ol []
                        (List.map
                            (\( occurrence, event ) ->
                                li [] [ viewOccurrence model.timezone occurrence event ]
                            )
                            occurrences
                        )
                    ]
            )
            (Events.dateTreeFromOccurrenceList model.timezone model.events)
        )
    ]


viewOccurrence : Time.Zone -> OccurrenceTime -> Event -> Html Msg
viewOccurrence zone occurrence event =
    let
        startTime =
            stringFromSimpleTime occurrence.startTime

        description =
            startTime ++ " - " ++ occurrence.location.name
    in
    div []
        [ p [] [ text event.name ]
        , p [] [ text description ]
        ]



-- View Helpers


stringFromDate : Date -> String
stringFromDate date =
    let
        day =
            Date.day date
                |> padInt

        month =
            Date.monthNumber date
                |> padInt

        year =
            Date.year date
                |> String.fromInt
    in
    day ++ "." ++ month ++ "." ++ year


stringFromSimpleTime : SimpleTime -> String
stringFromSimpleTime time =
    let
        hour =
            SimpleTime.hour time
                |> padInt

        minute =
            SimpleTime.minute time
                |> padInt
    in
    hour ++ ":" ++ minute


padInt : Int -> String
padInt n =
    if n < 10 then
        "0" ++ String.fromInt n

    else
        String.fromInt n
