module Main exposing (main)

import Browser
import Browser.Navigation as Browser
import Css exposing (auto, em, pct, pre, zero)
import Css.Global as Css
import Events exposing (Event, Events, Location, Occurrence)
import Html.Styled as Html exposing (Html, a, div, h1, h2, label, li, ol, p, text)
import Html.Styled.Attributes exposing (css, href, type_, value)
import Html.Styled.Events exposing (onClick, onInput)
import Http
import Json.Decode as Decode
import Pages.CreateEvent
import Pages.CreateLocation
import Pages.EditEvent
import Pages.EditLocation
import Pages.Overview
import Routes exposing (Route)
import Task
import Time
import Url exposing (Url)
import Utils.NaiveDateTime as Naive



-- Main


main =
    Browser.application
        { init = init
        , view = view
        , update = update
        , subscriptions = subscriptions
        , onUrlRequest = LinkClicked
        , onUrlChange = UrlChanged
        }



-- Model


type Model
    = Starting Browser.Key Route StartingModel
    | Error Browser.Key String
    | Loaded LoadedModel RouteModel
    | Loading LoadedModel RouteModel Route StartingModel


type StartingModel
    = LoadingToday
    | LoadingStore Naive.DateTime


type alias LoadedModel =
    { key : Browser.Key
    , today : Naive.DateTime
    , store : Events.Store
    }


keyFromModel : Model -> Browser.Key
keyFromModel model =
    case model of
        Starting key _ _ ->
            key

        Error key _ ->
            key

        Loaded { key } _ ->
            key

        Loading { key } _ _ _ ->
            key


type RouteModel
    = NotFound
    | Overview Pages.Overview.Model
    | CreateEvent Pages.CreateEvent.Model
    | EditEvent Pages.EditEvent.Model
    | CreateLocation Pages.CreateLocation.Model
    | EditLocation Pages.EditLocation.Model



-- I/O


init : () -> Url -> Browser.Key -> ( Model, Cmd Msg )
init flags url key =
    let
        route =
            Routes.toRoute url
    in
    ( Starting key route LoadingToday, fetchToday )


fetchToday : Cmd Msg
fetchToday =
    Task.perform FetchedToday Naive.now


initWith : Browser.Key -> Naive.DateTime -> Events.Store -> Route -> ( Model, Cmd Msg )
initWith key today store route =
    load (LoadedModel key today store) route


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.none



-- Update


type Msg
    = NoOp
    | FetchedToday Naive.DateTime
    | FetchedStore (Result Http.Error Events.Store)
    | LinkClicked Browser.UrlRequest
    | UrlChanged Url
    | CreateEventMsg Pages.CreateEvent.Msg
    | EditEventMsg Pages.EditEvent.Msg
    | CreateLocationMsg Pages.CreateLocation.Msg
    | EditLocationMsg Pages.EditLocation.Msg


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        NoOp ->
            ( model, Cmd.none )

        FetchedToday today ->
            case model of
                Starting key route LoadingToday ->
                    ( Starting key route (LoadingStore today), Events.fetchStore today FetchedStore )

                Loading loaded routeModel route LoadingToday ->
                    ( Loading loaded routeModel route (LoadingStore today), Events.fetchStore today FetchedStore )

                _ ->
                    ( model, Cmd.none )

        FetchedStore result ->
            case model of
                Starting key route (LoadingStore today) ->
                    case result of
                        Ok store ->
                            load (LoadedModel key today store) route

                        Err error ->
                            ( Error key (errorMessageFromHttpError error), Cmd.none )

                Loading loaded routeModel route (LoadingStore today) ->
                    case result of
                        Ok store ->
                            load (LoadedModel loaded.key today store) route

                        Err error ->
                            ( Error loaded.key (errorMessageFromHttpError error), Cmd.none )

                _ ->
                    ( model, Cmd.none )

        LinkClicked request ->
            case request of
                Browser.Internal url ->
                    let
                        key =
                            keyFromModel model
                    in
                    ( model, Browser.pushUrl key (Url.toString url) )

                Browser.External href ->
                    ( model, Browser.load href )

        UrlChanged url ->
            case model of
                Starting key _ loadState ->
                    ( Starting key (Routes.toRoute url) loadState, Cmd.none )

                Error key _ ->
                    init () url key

                Loaded loaded routeModel ->
                    ( Loading loaded routeModel (Routes.toRoute url) LoadingToday, fetchToday )

                Loading loaded routeModel _ loadState ->
                    ( Loading loaded routeModel (Routes.toRoute url) loadState, Cmd.none )

        CreateEventMsg subMsg ->
            let
                udpater routeModel =
                    case routeModel of
                        CreateEvent subModel ->
                            Pages.CreateEvent.update subMsg subModel
                                |> Tuple.mapBoth CreateEvent (Cmd.map CreateEventMsg)

                        _ ->
                            ( routeModel, Cmd.none )
            in
            updateLoaded udpater model

        EditEventMsg subMsg ->
            let
                udpater routeModel =
                    case routeModel of
                        EditEvent subModel ->
                            Pages.EditEvent.update subMsg subModel
                                |> Tuple.mapBoth EditEvent (Cmd.map EditEventMsg)

                        _ ->
                            ( routeModel, Cmd.none )
            in
            updateLoaded udpater model

        CreateLocationMsg subMsg ->
            let
                udpater routeModel =
                    case routeModel of
                        CreateLocation subModel ->
                            Pages.CreateLocation.update subMsg subModel
                                |> Tuple.mapBoth CreateLocation (Cmd.map CreateLocationMsg)

                        _ ->
                            ( routeModel, Cmd.none )
            in
            updateLoaded udpater model

        EditLocationMsg subMsg ->
            let
                udpater routeModel =
                    case routeModel of
                        EditLocation subModel ->
                            Pages.EditLocation.update subMsg subModel
                                |> Tuple.mapBoth EditLocation (Cmd.map EditLocationMsg)

                        _ ->
                            ( routeModel, Cmd.none )
            in
            updateLoaded udpater model


load : LoadedModel -> Route -> ( Model, Cmd Msg )
load loaded route =
    case route of
        Routes.NotFound ->
            ( wrapModel loaded (\_ -> NotFound) (), Cmd.none )

        Routes.Overview ->
            ( wrapModel loaded Overview <| Pages.Overview.init loaded.store, Cmd.none )

        Routes.CreateEvent ->
            wrap loaded CreateEvent CreateEventMsg <| Pages.CreateEvent.init loaded.key loaded.today loaded.store

        Routes.EditEvent rawId ->
            wrap loaded EditEvent EditEventMsg <| Pages.EditEvent.init loaded.today loaded.store rawId

        Routes.CreateLocation ->
            ( wrapModel loaded CreateLocation <| Pages.CreateLocation.init loaded.key, Cmd.none )

        Routes.EditLocation rawId ->
            wrap loaded EditLocation EditLocationMsg <| Pages.EditLocation.init loaded.store rawId


wrap : LoadedModel -> (model -> RouteModel) -> (msg -> Msg) -> ( model, Cmd msg ) -> ( Model, Cmd Msg )
wrap loaded wrapMdl wrapMsg ( model, msg ) =
    ( wrapModel loaded wrapMdl model, Cmd.map wrapMsg msg )


wrapModel : LoadedModel -> (model -> RouteModel) -> model -> Model
wrapModel loaded wrapMdl model =
    Loaded (LoadedModel loaded.key loaded.today loaded.store) <| wrapMdl model


updateLoaded : (RouteModel -> ( RouteModel, Cmd Msg )) -> Model -> ( Model, Cmd Msg )
updateLoaded updater model =
    case model of
        Starting _ _ _ ->
            ( model, Cmd.none )

        Error _ _ ->
            ( model, Cmd.none )

        Loaded loaded routeModel ->
            updater routeModel
                |> Tuple.mapFirst (Loaded loaded)

        Loading loaded routeModel route loadState ->
            updater routeModel
                |> Tuple.mapFirst (\newRouteModel -> Loading loaded newRouteModel route loadState)


errorMessageFromHttpError : Http.Error -> String
errorMessageFromHttpError httpError =
    case httpError of
        Http.BadUrl url ->
            "The URL " ++ url ++ " is invalid."

        Http.Timeout ->
            "The request timed out."

        Http.NetworkError ->
            "A network error occurred."

        Http.BadStatus status ->
            "The response had status " ++ String.fromInt status ++ "."

        Http.BadBody error ->
            "The response's body was invalid:\n" ++ error



-- View


view : Model -> Browser.Document Msg
view model =
    let
        styledBody =
            let
                render loaded =
                    case loaded of
                        NotFound ->
                            viewNotFound

                        Overview subModel ->
                            Pages.Overview.view subModel

                        CreateEvent subModel ->
                            Pages.CreateEvent.view subModel
                                |> List.map (Html.map CreateEventMsg)

                        EditEvent subModel ->
                            Pages.EditEvent.view subModel
                                |> List.map (Html.map EditEventMsg)

                        CreateLocation subModel ->
                            Pages.CreateLocation.view subModel
                                |> List.map (Html.map CreateLocationMsg)

                        EditLocation subModel ->
                            Pages.EditLocation.view subModel
                                |> List.map (Html.map EditLocationMsg)
            in
            case model of
                Starting _ _ _ ->
                    viewLoading

                Error _ error ->
                    viewError error

                Loaded _ routeModel ->
                    render routeModel

                Loading _ routeModel _ _ ->
                    loadIndicator
                        ++ render routeModel

        mainStyle =
            Css.global
                [ Css.body
                    [ Css.fontFamily Css.sansSerif
                    , Css.margin2 zero auto
                    , Css.maxWidth (em 64)
                    , Css.padding (em 1.5)
                    ]
                ]
    in
    { title = "Lindy Hop Aachen Admin"
    , body = List.map Html.toUnstyled (mainStyle :: styledBody)
    }


viewLoading : List (Html Msg)
viewLoading =
    [ text "Lädt..." ]


loadIndicator : List (Html msg)
loadIndicator =
    [ div
        [ css
            [ Css.position Css.absolute
            , Css.top zero
            , Css.left zero
            , Css.height (em 0.1)
            , Css.width (Css.vw 100)
            , Css.backgroundColor (Css.rgb 200 150 0)
            ]
        ]
        []
    ]


viewError : String -> List (Html Msg)
viewError error =
    [ p [] [ text "Es gab einen Fehler beim Laden." ]
    , p [ css [ Css.fontFamily Css.monospace, Css.whiteSpace pre ] ] [ text error ]
    ]


viewNotFound : List (Html Msg)
viewNotFound =
    [ text "Die Seite konnte nicht gefunden werden." ]
