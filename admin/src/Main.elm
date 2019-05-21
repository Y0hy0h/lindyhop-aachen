module Main exposing (main)

import Browser
import Browser.Navigation as Browser
import Css exposing (auto, em, zero)
import Css.Global as Css
import Events exposing (Event, Events, Location, Occurrence)
import Html.Styled as Html exposing (Html, a, div, h1, h2, label, li, ol, p, text)
import Html.Styled.Attributes exposing (href, type_, value)
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
    = Loaded Browser.Key RouteModel
    | Loading Browser.Key RouteModel RouteLoadModel


keyFromModel : Model -> Browser.Key
keyFromModel model =
    case model of
        Loaded key _ ->
            key

        Loading key _ _ ->
            key


loadedFromModel : Model -> RouteModel
loadedFromModel model =
    case model of
        Loaded _ routeModel ->
            routeModel

        Loading _ routeModel _ ->
            routeModel


type RouteModel
    = LoadingRoute
    | ErrorLoading
    | NotFound
    | Overview Pages.Overview.Model
    | CreateEvent Pages.CreateEvent.Model
    | EditEvent Pages.EditEvent.Model
    | CreateLocation Pages.CreateLocation.Model
    | EditLocation Pages.EditLocation.Model


type RouteLoadModel
    = OverviewLoad Pages.Overview.LoadModel
    | CreateEventLoad Pages.CreateEvent.LoadModel
    | EditEventLoad Pages.EditEvent.LoadModel
    | EditLocationLoad Pages.EditLocation.LoadModel



-- I/O


init : () -> Url -> Browser.Key -> ( Model, Cmd Msg )
init flags url key =
    let
        route =
            Routes.toRoute url
    in
    initWith key route


initWith : Browser.Key -> Route -> ( Model, Cmd Msg )
initWith key route =
    load (Loaded key LoadingRoute) route


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.none



-- Update


type Msg
    = NoOp
    | LinkClicked Browser.UrlRequest
    | UrlChanged Url
    | OverviewLoadMsg Pages.Overview.LoadMsg
    | CreateEventLoadMsg Pages.CreateEvent.LoadMsg
    | CreateEventMsg Pages.CreateEvent.Msg
    | EditEventLoadMsg Pages.EditEvent.LoadMsg
    | EditEventMsg Pages.EditEvent.Msg
    | CreateLocationMsg Pages.CreateLocation.Msg
    | EditLocationLoadMsg Pages.EditLocation.LoadMsg
    | EditLocationMsg Pages.EditLocation.Msg


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        NoOp ->
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
            load model (Routes.toRoute url)

        OverviewLoadMsg subMsg ->
            case model of
                Loading key loaded (OverviewLoad subModel) ->
                    case Pages.Overview.updateLoad subMsg subModel of
                        Ok newSubModel ->
                            ( Loaded key (Overview newSubModel), Cmd.none )

                        Err error ->
                            ( Loaded key ErrorLoading, Cmd.none )

                _ ->
                    ( model, Cmd.none )

        CreateEventLoadMsg subMsg ->
            case model of
                Loading key loaded (CreateEventLoad subModel) ->
                    case Pages.CreateEvent.updateLoad subMsg subModel of
                        Ok newSubModel ->
                            ( Loaded key (CreateEvent newSubModel), Cmd.none )

                        Err error ->
                            ( Loaded key ErrorLoading, Cmd.none )

                _ ->
                    ( model, Cmd.none )

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

        EditEventLoadMsg subMsg ->
            case model of
                Loading key loaded (EditEventLoad subModel) ->
                    case Pages.EditEvent.updateLoad subMsg subModel of
                        Ok newSubModel ->
                            ( Loaded key (EditEvent newSubModel), Cmd.none )

                        Err error ->
                            ( Loaded key ErrorLoading, Cmd.none )

                _ ->
                    ( model, Cmd.none )

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

        EditLocationLoadMsg subMsg ->
            case model of
                Loading key loaded (EditLocationLoad subModel) ->
                    case Pages.EditLocation.updateLoad subMsg subModel of
                        Ok newSubModel ->
                            ( Loaded key (EditLocation newSubModel), Cmd.none )

                        Err error ->
                            ( Loaded key ErrorLoading, Cmd.none )

                _ ->
                    ( model, Cmd.none )

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


load : Model -> Route -> ( Model, Cmd Msg )
load model route =
    let
        key =
            keyFromModel model
    in
    case route of
        Routes.NotFound ->
            ( Loaded key <| NotFound, Cmd.none )

        Routes.Overview ->
            Pages.Overview.init
                |> wrapLoadModel model OverviewLoad OverviewLoadMsg

        Routes.CreateEvent ->
            Pages.CreateEvent.init key
                |> wrapLoadModel model CreateEventLoad CreateEventLoadMsg

        Routes.EditEvent rawId ->
            Pages.EditEvent.init rawId
                |> wrapLoadModel model EditEventLoad EditEventLoadMsg

        Routes.CreateLocation ->
            ( Loaded key <| CreateLocation <| Pages.CreateLocation.init key, Cmd.none )

        Routes.EditLocation rawId ->
            Pages.EditLocation.init rawId
                |> wrapLoadModel model EditLocationLoad EditLocationLoadMsg


wrapLoadModel : Model -> (subModel -> RouteLoadModel) -> (subLoadMsg -> msg) -> ( subModel, Cmd subLoadMsg ) -> ( Model, Cmd msg )
wrapLoadModel model wrapper loadMsgWrapper updateTuple =
    let
        key =
            keyFromModel model

        loaded =
            loadedFromModel model
    in
    Tuple.mapBoth
        (\subModel -> Loading key loaded (wrapper subModel))
        (Cmd.map loadMsgWrapper)
        updateTuple


updateLoaded : (RouteModel -> ( RouteModel, Cmd Msg )) -> Model -> ( Model, Cmd Msg )
updateLoaded updater model =
    case model of
        Loaded key loaded ->
            updater loaded |> Tuple.mapFirst (Loaded key)

        Loading key loaded loading ->
            updater loaded |> Tuple.mapFirst (\newLoaded -> Loading key newLoaded loading)



-- View


view : Model -> Browser.Document Msg
view model =
    let
        styledBody =
            case loadedFromModel model of
                LoadingRoute ->
                    viewLoading

                ErrorLoading ->
                    viewErrorLoading

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
    [ text "Loading..." ]


viewErrorLoading : List (Html Msg)
viewErrorLoading =
    [ text "There was an error while loading the app." ]


viewNotFound : List (Html Msg)
viewNotFound =
    [ text "Not found." ]
