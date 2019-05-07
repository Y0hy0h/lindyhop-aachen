module Pages.EditLocation exposing
    ( InputMsg
    , LoadModel
    , LoadMsg
    , LocationInput
    , Model
    , Msg
    , fromEvents
    , init
    , locationFromInputs
    , update
    , updateInputs
    , updateLoad
    , view
    , viewEditLocation
    )

import Css exposing (row)
import Events exposing (Event, Events, Location, Occurrence)
import Html.Styled as Html exposing (Html, a, div, input, label, li, ol, p, text, textarea)
import Html.Styled.Attributes exposing (css, href, type_, value)
import Html.Styled.Events exposing (onInput)
import Http
import IdDict exposing (Id)
import Json.Encode as Encode
import List.Extra as List
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
    { locationId : Id Location
    , location : Location
    , inputs : LocationInput
    }


type alias LocationInput =
    { name : In String
    , address : In String
    }


inputsFromLocation : Location -> LocationInput
inputsFromLocation location =
    { name = inputString location.name
    , address = inputString location.address
    }


locationFromInputs : LocationInput -> Maybe Location
locationFromInputs inputs =
    Maybe.map2
        Location
        (extract inputs.name)
        (extract inputs.address)


type alias LoadModel =
    { rawId : String
    }


init : String -> ( LoadModel, Cmd LoadMsg )
init rawId =
    let
        fetchEvents =
            Events.fetchEvents FetchedEvents
    in
    ( LoadModel rawId, fetchEvents )


fromEvents : String -> Events.Store -> Maybe Model
fromEvents rawId store =
    let
        locations =
            Events.locations store
    in
    IdDict.validate rawId locations
        |> Maybe.map
            (\id ->
                let
                    location =
                        IdDict.get id locations

                    inputs =
                        inputsFromLocation location
                in
                Model id location inputs
            )


type LoadMsg
    = FetchedEvents (Result Http.Error Events.Store)


type LoadError
    = Http Http.Error
    | InvalidId String


updateLoad : LoadMsg -> LoadModel -> Result LoadError Model
updateLoad msg model =
    case msg of
        FetchedEvents result ->
            Result.mapError Http result
                |> Result.andThen
                    (\events ->
                        fromEvents model.rawId events
                            |> Result.fromMaybe (InvalidId model.rawId)
                    )


type Msg
    = Input InputMsg
    | ClickedSave
    | SaveFinished (Result Http.Error ())
    | ClickedDelete
    | DeleteFinished (Result Http.Error Location)


type InputMsg
    = InputName String
    | InputAddress String


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        Input inputMsg ->
            ( { model | inputs = updateInputs inputMsg model.inputs }, Cmd.none )

        ClickedSave ->
            let
                cmd =
                    case locationFromInputs model.inputs of
                        Just location ->
                            Events.updateLocation model.locationId location SaveFinished

                        Nothing ->
                            Cmd.none
            in
            ( model, cmd )

        SaveFinished result ->
            ( model, Cmd.none )

        ClickedDelete ->
            ( model, Events.deleteLocation model.locationId DeleteFinished )

        DeleteFinished result ->
            ( model, Cmd.none )


updateInputs : InputMsg -> LocationInput -> LocationInput
updateInputs msg location =
    let
        setInput new input =
            updateInput (\_ -> new) input
    in
    case msg of
        InputName newName ->
            { location | name = setInput newName location.name }

        InputAddress newAddress ->
            { location | address = setInput newAddress location.address }


updateLocation : Model -> (LocationInput -> LocationInput) -> Model
updateLocation model locationUpdater =
    let
        location =
            model.inputs

        newLocation =
            locationUpdater location
    in
    { model | inputs = newLocation }


view : Model -> List (Html Msg)
view model =
    [ Utils.breadcrumbs [ Routes.Overview ] (Routes.EditLocation <| IdDict.encodeIdForUrl model.locationId)
    ]
        ++ (List.map (Html.map Input) <| viewEditLocation model.inputs)
        ++ [ div [ css [ Css.displayFlex, Css.flexDirection row ] ]
                [ let
                    options =
                        { enabledness =
                            if changed model then
                                Utils.Enabled

                            else
                                Utils.Disabled
                        }
                  in
                  Utils.buttonWithOptions options "Speichern" ClickedSave
                , Utils.button "Löschen" ClickedDelete
                ]
           ]


viewEditLocation : LocationInput -> List (Html InputMsg)
viewEditLocation inputs =
    [ Utils.fields
        [ viewInputText "Bezeichnung" inputs.name InputName
        , viewTextArea "Adresse" inputs.address InputAddress
        ]
    ]


changed : Model -> Bool
changed model =
    locationFromInputs model.inputs
        |> Maybe.map (\newLocation -> newLocation /= model.location)
        |> Maybe.withDefault False
