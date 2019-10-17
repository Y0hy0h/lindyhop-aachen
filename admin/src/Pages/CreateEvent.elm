module Pages.CreateEvent exposing
    ( Model
    , Msg
    , init
    , update
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
    , today : Naive.DateTime
    , inputs : Edit.InputModel
    , locations : Locations
    }


init : Browser.Key -> Naive.DateTime -> Events.Store -> ( Model, Cmd Msg )
init key today store =
    let
        locations =
            Events.locations store

        ( batchAddModel, batchAddMsg ) =
            Edit.initBatchAddModel locations

        inputs =
            { eventInputs =
                { title = Utils.inputString ""
                , teaser = Utils.inputString ""
                , description = Utils.inputString ""
                , occurrences = []
                }
            , batchAdd = batchAddModel
            }
    in
    ( Model key today inputs locations, Cmd.map Input batchAddMsg )


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
                    case Edit.eventFromInputs model.locations model.inputs.eventInputs of
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
        ++ [ Utils.bottomToolbar
                [ Utils.button "Speichern" ClickedSave
                ]
           ]
