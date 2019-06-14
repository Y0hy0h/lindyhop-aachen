module Pages.CreateLocation exposing
    ( Model
    , Msg
    , init
    , update
    , view
    )

import Browser
import Browser.Navigation as Browser
import Css exposing (row)
import Events exposing (Event, Events, Location, Occurrence)
import Html.Styled as Html exposing (Html, a, div, input, label, li, ol, p, text, textarea)
import Html.Styled.Attributes exposing (css, href, type_, value)
import Html.Styled.Events exposing (onInput)
import Http
import IdDict exposing (Id)
import Json.Encode as Encode
import List.Extra as List
import Pages.EditLocation as Edit
import Pages.Utils as Utils
    exposing
        ( In
        , Input
        , extract
        , inputString
        , updateInput
        , viewDateTimeInput
        , viewInputNumber
        , viewInputText
        , viewTextArea
        )
import Parser
import Routes
import Time
import Utils.NaiveDateTime as Naive
import Utils.TimeFormat as TimeFormat


type alias Model =
    { key : Browser.Key
    , inputs : Edit.LocationInput
    }


init : Browser.Key -> Model
init key =
    let
        inputs =
            { name = Utils.inputString ""
            , address = Utils.inputString ""
            }
    in
    Model key inputs


type Msg
    = Input Edit.InputMsg
    | ClickedSave
    | SaveFinished (Result Http.Error IdDict.UnsafeId)


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        Input inputMsg ->
            ( { model | inputs = Edit.updateInputs inputMsg model.inputs }, Cmd.none )

        ClickedSave ->
            let
                cmd =
                    case Edit.locationFromInputs model.inputs of
                        Just location ->
                            Events.createLocation location SaveFinished

                        Nothing ->
                            Cmd.none
            in
            ( model, cmd )

        SaveFinished result ->
            case result of
                Ok id ->
                    ( model, Browser.pushUrl model.key (Routes.toRelativeUrl <| Routes.EditLocation id) )

                Err _ ->
                    ( model, Cmd.none )


view : Model -> List (Html Msg)
view model =
    [ Utils.breadcrumbs [ Routes.Overview ] Routes.CreateLocation
    ]
        ++ (List.map (Html.map Input) <| Edit.viewEditLocation model.inputs)
        ++ [ Utils.button "Speichern" ClickedSave
           ]
