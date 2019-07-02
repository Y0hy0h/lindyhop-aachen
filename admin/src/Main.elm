module Main exposing (main)

import Browser
import Browser.Navigation as Browser
import Css exposing (auto, em, pre, zero)
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
    = Starting Browser.Key Route
    | Loaded Browser.Key Naive.DateTime RouteModel
    | Loading Browser.Key Naive.DateTime RouteModel RouteLoadModel


keyFromModel : Model -> Browser.Key
keyFromModel model =
    case model of
        Starting key _ ->
            key

        Loaded key _ _ ->
            key

        Loading key _ _ _ ->
            key


type RouteModel
    = LoadingRoute
    | ErrorLoading String
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

        fetchToday =
            Naive.now
    in
    ( Starting key route, Task.perform FetchedToday fetchToday )


initWith : Browser.Key -> Naive.DateTime -> Route -> ( Model, Cmd Msg )
initWith key today route =
    load today (Loaded key today LoadingRoute) route


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.none



-- Update


type Msg
    = NoOp
    | FetchedToday Naive.DateTime
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

        FetchedToday today ->
            case model of
                Starting key route ->
                    load today (Loaded key today LoadingRoute) route

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
                Starting key route ->
                    ( Starting key (Routes.toRoute url), Cmd.none )

                Loaded _ today _ ->
                    load today model (Routes.toRoute url)

                Loading _ today _ _ ->
                    load today model (Routes.toRoute url)

        OverviewLoadMsg subMsg ->
            case model of
                Loading key today loaded (OverviewLoad subModel) ->
                    case Pages.Overview.updateLoad subMsg subModel of
                        Ok newSubModel ->
                            ( Loaded key today (Overview newSubModel), Cmd.none )

                        Err error ->
                            ( Loaded key today (ErrorLoading <| errorMessageFromHttpError error), Cmd.none )

                _ ->
                    ( model, Cmd.none )

        CreateEventLoadMsg subMsg ->
            case model of
                Loading key today loaded (CreateEventLoad subModel) ->
                    case Pages.CreateEvent.updateLoad subMsg subModel of
                        Ok ( newSubModel, newSubMsg ) ->
                            ( Loaded key today (CreateEvent newSubModel), Cmd.map CreateEventMsg newSubMsg )

                        Err error ->
                            ( Loaded key today (ErrorLoading <| errorMessageFromHttpError error), Cmd.none )

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
                Loading key today loaded (EditEventLoad subModel) ->
                    case Pages.EditEvent.updateLoad subMsg subModel of
                        Ok ( newSubModel, newSubMsg ) ->
                            ( Loaded key today (EditEvent newSubModel), Cmd.map EditEventMsg newSubMsg )

                        Err error ->
                            let
                                errorMessage =
                                    case error of
                                        Pages.EditEvent.Http httpError ->
                                            errorMessageFromHttpError httpError

                                        Pages.EditEvent.InvalidId id ->
                                            "The id " ++ id ++ " is invalid."
                            in
                            ( Loaded key today (ErrorLoading errorMessage), Cmd.none )

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
                Loading key today loaded (EditLocationLoad subModel) ->
                    case Pages.EditLocation.updateLoad subMsg subModel of
                        Ok newSubModel ->
                            ( Loaded key today (EditLocation newSubModel), Cmd.none )

                        Err error ->
                            let
                                errorMessage =
                                    case error of
                                        Pages.EditLocation.Http httpError ->
                                            errorMessageFromHttpError httpError

                                        Pages.EditLocation.InvalidId id ->
                                            "The id " ++ id ++ " is invalid."
                            in
                            ( Loaded key today (ErrorLoading errorMessage), Cmd.none )

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


load : Naive.DateTime -> Model -> Route -> ( Model, Cmd Msg )
load today model route =
    let
        key =
            keyFromModel model
    in
    case route of
        Routes.NotFound ->
            ( Loaded key today <| NotFound, Cmd.none )

        Routes.Overview ->
            Pages.Overview.init today
                |> wrapLoadModel model OverviewLoad OverviewLoadMsg

        Routes.CreateEvent ->
            Pages.CreateEvent.init key today
                |> wrapLoadModel model CreateEventLoad CreateEventLoadMsg

        Routes.EditEvent rawId ->
            Pages.EditEvent.init today rawId
                |> wrapLoadModel model EditEventLoad EditEventLoadMsg

        Routes.CreateLocation ->
            ( Loaded key today <| CreateLocation <| Pages.CreateLocation.init key, Cmd.none )

        Routes.EditLocation rawId ->
            Pages.EditLocation.init today rawId
                |> wrapLoadModel model EditLocationLoad EditLocationLoadMsg


wrapLoadModel : Model -> (subModel -> RouteLoadModel) -> (subLoadMsg -> msg) -> ( subModel, Cmd subLoadMsg ) -> ( Model, Cmd msg )
wrapLoadModel model wrapper loadMsgWrapper updateTuple =
    let
        wrap routeModel today =
            let
                key =
                    keyFromModel model
            in
            Tuple.mapBoth
                (\subModel -> Loading key today routeModel (wrapper subModel))
                (Cmd.map loadMsgWrapper)
                updateTuple
    in
    case model of
        Starting _ _ ->
            ( model, Cmd.none )

        Loaded key today routeModel ->
            wrap routeModel today

        Loading key today routeModel loading ->
            wrap routeModel today


updateLoaded : (RouteModel -> ( RouteModel, Cmd Msg )) -> Model -> ( Model, Cmd Msg )
updateLoaded updater model =
    case model of
        Starting _ _ ->
            ( model, Cmd.none )

        Loaded key today loaded ->
            updater loaded |> Tuple.mapFirst (Loaded key today)

        Loading key today loaded loading ->
            updater loaded |> Tuple.mapFirst (\newLoaded -> Loading key today newLoaded loading)


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
                        LoadingRoute ->
                            viewLoading

                        ErrorLoading error ->
                            viewErrorLoading error

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
                Starting _ _ ->
                    viewLoading

                Loaded _ _ loaded ->
                    render loaded

                Loading _ _ loaded _ ->
                    render loaded

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


viewErrorLoading : String -> List (Html Msg)
viewErrorLoading error =
    [ p [] [ text "There was an error while loading the app." ]
    , p [ css [ Css.fontFamily Css.monospace, Css.whiteSpace pre ] ] [ text error ]
    ]


viewNotFound : List (Html Msg)
viewNotFound =
    [ text "Not found." ]
