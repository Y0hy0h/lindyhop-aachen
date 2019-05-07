module Pages.CreateEvent exposing
    ( LoadModel
    , LoadMsg
    , Model
    , Msg
    , fromEvents
    , init
    , update
    , updateLoad
    , view
    )

import Browser
import Browser.Navigation as Browser
import Css exposing (em, flexStart, row, zero)
import Css.Global as Css
import Events exposing (Event, Location, Locations, Occurrence)
import Html.Styled as Html exposing (Html, a, div, h2, input, label, li, ol, p, text, textarea)
import Html.Styled.Attributes exposing (css, href, type_, value)
import Html.Styled.Events exposing (onInput)
import Http
import IdDict exposing (Id)
import Json.Encode as Encode
import List.Extra as List
import Maybe.Extra as Maybe
import Pages.EditEvent as Edit
import Pages.Utils as Utils
    exposing
        ( In
        , Input
        , extract
        , fields
        , inputDateTime
        , inputString
        , labeled
        , updateInput
        , viewDateTimeInput
        , viewInputNumber
        , viewInputText
        , viewTextArea
        )
import Parser
import Routes
import Time
import Utils.NaiveDateTime as Naive exposing (Duration)
import Utils.TimeFormat as TimeFormat
import Utils.Validate as Validate exposing (Validator)


type alias Model =
    { key : Browser.Key
    , inputs : Edit.EventInput
    , locations : Locations
    }


type alias LoadModel =
    { key : Browser.Key }


init : Browser.Key -> ( LoadModel, Cmd LoadMsg )
init key =
    let
        fetchEvents =
            Events.fetchEvents FetchedEvents
    in
    ( LoadModel key, fetchEvents )


fromEvents : Browser.Key -> Events.Store -> Model
fromEvents key store =
    let
        inputs =
            { name = Utils.inputString ""
            , teaser = Utils.inputString ""
            , description = Utils.inputString ""
            , occurrences = []
            }

        locations =
            Events.locations store
    in
    Model key inputs locations


type LoadMsg
    = FetchedEvents (Result Http.Error Events.Store)


updateLoad : LoadMsg -> LoadModel -> Result Http.Error Model
updateLoad msg model =
    case msg of
        FetchedEvents result ->
            Result.map
                (\events ->
                    fromEvents model.key events
                )
                result


type Msg
    = Input Edit.InputMsg
    | ClickedSave
    | SaveFinished (Result Http.Error IdDict.UnsafeId)


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        Input inputMsg ->
            ( { model | inputs = Edit.updateInputs model.locations inputMsg model.inputs }, Cmd.none )

        ClickedSave ->
            let
                cmd =
                    case Edit.eventFromInputs model.locations model.inputs of
                        Just event ->
                            Events.createEvent event SaveFinished

                        Nothing ->
                            Cmd.none
            in
            ( model, cmd )

        SaveFinished result ->
            case result of
                Ok id ->
                    ( model, Browser.pushUrl model.key (Routes.toRelativeUrl <| Routes.EditEvent id) )

                Err _ ->
                    ( model, Cmd.none )


view : Model -> List (Html Msg)
view model =
    [ Utils.breadcrumbs [ Routes.Overview ] Routes.CreateEvent ]
        ++ (List.map (Html.map Input) <| Edit.viewEditEvent model.locations model.inputs)
        ++ [ Utils.button "Speichern" ClickedSave
           ]
