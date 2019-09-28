module Pages.EditEvent exposing
    ( DisplayStatus(..)
    , EventInput
    , InputModel
    , InputMsg
    , Model
    , Msg
    , eventFromInputs
    , init
    , initBatchAddModel
    , update
    , updateInputs
    , view
    , viewEditEvent
    )

import Css exposing (em, flexStart, int, px, row, vh, zero)
import Css.Global as Css
import Date exposing (Date)
import Events exposing (Event, Location, Locations, Occurrence)
import Html.Styled as Html exposing (Html, a, details, div, h2, h3, input, label, li, ol, p, summary, text, textarea)
import Html.Styled.Attributes exposing (css, href, type_, value)
import Html.Styled.Events exposing (onInput)
import Http
import IdDict exposing (Id)
import Json.Encode as Encode
import List.Extra as List
import Maybe.Extra as Maybe
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
        , viewTimeInput
        )
import Parser
import Routes
import Time
import Utils.MultiselectCalendar as MultiselectCalendar
import Utils.NaiveDateTime as Naive exposing (Duration)
import Utils.TimeFormat as TimeFormat
import Utils.Validate as Validate exposing (Validator)


type Model
    = Valid ModelData
    | Invalid String


type alias ModelData =
    { eventId : Id Event
    , event : Event
    , today : Naive.DateTime
    , inputs : InputModel
    , locations : Locations
    }


type alias InputModel =
    { eventInputs : EventInput
    , batchAdd : BatchAddModel
    }


type alias BatchAddModel =
    { inputs : BatchOccurrenceInput
    , dates : MultiselectCalendar.Model
    }


type alias EventInput =
    { title : In String
    , teaser : In String
    , description : In String
    , occurrences : List OccurrenceInput
    }


type DisplayStatus
    = Shown
    | Hidden


toggle : DisplayStatus -> DisplayStatus
toggle state =
    case state of
        Shown ->
            Hidden

        Hidden ->
            Shown


type alias OccurrenceInput =
    { start : Input { date : String, time : String } Naive.DateTime
    , duration : In Duration
    , locationId : In (Id Location)
    }


type alias BatchOccurrenceInput =
    { start : In Naive.Time
    , duration : In Duration
    , locationId : In (Id Location)
    }


initBatchAddModel : Locations -> ( BatchAddModel, Cmd InputMsg )
initBatchAddModel locations =
    let
        ( calendarModel, calendarMsg ) =
            MultiselectCalendar.init []
    in
    ( { inputs = emptyBatchOccurrenceInput locations
      , dates = calendarModel
      }
    , Cmd.map (InputBatchAdd << BatchMultiselectCalendarMsg) calendarMsg
    )


emptyBatchOccurrenceInput : Locations -> BatchOccurrenceInput
emptyBatchOccurrenceInput locations =
    { start = Utils.buildInput "" Utils.timeValidator
    , duration = Utils.buildInput "" durationValidator
    , locationId = Utils.buildInput "" (locationIdValidator locations)
    }


eventFromInputs : Locations -> EventInput -> Maybe Event
eventFromInputs locs inputs =
    let
        maybeOccurrences =
            Maybe.combine (List.map (occurrenceFromInput locs) inputs.occurrences)
    in
    Maybe.map4 Event
        (extract inputs.title)
        (extract inputs.teaser)
        (extract inputs.description)
        maybeOccurrences


occurrenceFromInput : Locations -> OccurrenceInput -> Maybe Occurrence
occurrenceFromInput locs input =
    Maybe.map3
        Occurrence
        (extract input.start)
        (extract input.duration)
        (extract input.locationId)


inputsFromEvent : Locations -> Event -> EventInput
inputsFromEvent locations event =
    let
        inputFromOccurrence : Occurrence -> OccurrenceInput
        inputFromOccurrence occurrence =
            { start = inputDateTime occurrence.start
            , duration = inputDuration occurrence.duration
            , locationId = inputLocationId locations occurrence.locationId
            }
    in
    { title = inputString event.title
    , teaser = inputString event.teaser
    , description = inputString event.description
    , occurrences = List.map inputFromOccurrence event.occurrences
    }


inputDuration : Duration -> In Duration
inputDuration duration =
    let
        value =
            Naive.asMinutes duration |> String.fromInt
    in
    Utils.buildInput value durationValidator


durationValidator : Validator String Duration
durationValidator =
    Validate.from
        (\raw ->
            String.toInt raw
                |> Result.fromMaybe [ "Bitte eine Zahl eingeben." ]
                |> Result.andThen
                    (Naive.minutes
                        >> Result.fromMaybe [ "Die Dauer darf nicht negativ sein." ]
                    )
        )


inputLocationId : Locations -> Id Location -> In (Id Location)
inputLocationId locations id =
    let
        value =
            IdDict.encodeIdForUrl id
    in
    Utils.buildInput value (locationIdValidator locations)


locationIdValidator : Locations -> Validator String (Id Location)
locationIdValidator locations =
    Validate.from
        (\raw ->
            IdDict.validate raw locations
                |> Result.fromMaybe [ "Der gewählte Ort konnte nicht gefunden werden." ]
        )


init : Naive.DateTime -> Events.Store -> String -> ( Model, Cmd Msg )
init today store rawId =
    let
        events =
            Events.events store

        locations =
            Events.locations store
    in
    IdDict.validate rawId events
        |> Maybe.map
            (\id ->
                let
                    event =
                        IdDict.get id events

                    ( inputModel, inputMsg ) =
                        inputModelFromEvents locations event

                    loadedModel =
                        { eventId = id
                        , event = event
                        , today = today
                        , locations = locations
                        , inputs = inputModel
                        }
                in
                ( Valid loadedModel, Cmd.map Input inputMsg )
            )
        |> Maybe.withDefault ( Invalid rawId, Cmd.none )


inputModelFromEvents : Locations -> Event -> ( InputModel, Cmd InputMsg )
inputModelFromEvents locations event =
    let
        ( batchAddModel, batchAddMsg ) =
            initBatchAddModel locations

        model =
            { eventInputs = inputsFromEvent locations event
            , batchAdd = batchAddModel
            }
    in
    ( model, batchAddMsg )


type Msg
    = Input InputMsg
    | ClickedSave
    | SaveFinished (Result Http.Error ())
    | ClickedDelete
    | DeleteFinished (Result Http.Error Event)


type InputMsg
    = InputEvent EventMsg
    | InputBatchAdd BatchAddMsg
    | BatchAddRequested


type EventMsg
    = InputName String
    | InputTeaser String
    | InputDescription String
    | InputOccurrence Int OccurrenceMsg
    | AddOccurrence


type OccurrenceMsg
    = InputStartDate String
    | InputStartTime String
    | InputDuration String
    | InputLocationId String
    | InputClickedDelete


type BatchAddMsg
    = BatchMultiselectCalendarMsg MultiselectCalendar.Msg
    | BatchAddInputMsg BatchAddInputMsg


type BatchAddInputMsg
    = BatchInputStartTime String
    | BatchInputDuration String
    | BatchInputLocationId String


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case model of
        Valid data ->
            updateModelData msg data |> Tuple.mapFirst Valid

        Invalid _ ->
            ( model, Cmd.none )


updateModelData : Msg -> ModelData -> ( ModelData, Cmd Msg )
updateModelData msg model =
    case msg of
        Input inputMsg ->
            let
                newModel =
                    { model | inputs = updateInputs model.locations inputMsg model.inputs }
            in
            ( newModel, Cmd.none )

        ClickedSave ->
            let
                cmd =
                    case eventFromInputs model.locations model.inputs.eventInputs of
                        Just event ->
                            Events.updateEvent model.today model.eventId event SaveFinished

                        Nothing ->
                            Cmd.none
            in
            ( model, cmd )

        SaveFinished result ->
            ( model, Cmd.none )

        ClickedDelete ->
            ( model, Events.deleteEvent model.locations model.eventId DeleteFinished )

        DeleteFinished result ->
            ( model, Cmd.none )


updateInputs : Locations -> InputMsg -> InputModel -> InputModel
updateInputs locations msg inputs =
    case msg of
        InputEvent eventMsg ->
            { inputs
                | eventInputs = updateEventInputs locations eventMsg inputs.eventInputs
            }

        InputBatchAdd batchAddMsg ->
            { inputs
                | batchAdd = updateBatchAdd locations batchAddMsg inputs.batchAdd
            }

        BatchAddRequested ->
            let
                occurrenceInputs : List OccurrenceInput
                occurrenceInputs =
                    List.map
                        (\date ->
                            let
                                start : Input { date : String, time : String } Naive.DateTime
                                start =
                                    Utils.buildInput
                                        { date = Date.toIsoString date
                                        , time = Utils.getRaw inputs.batchAdd.inputs.start
                                        }
                                        Utils.dateTimeValidator
                            in
                            OccurrenceInput start inputs.batchAdd.inputs.duration inputs.batchAdd.inputs.locationId
                        )
                        (MultiselectCalendar.selected inputs.batchAdd.dates)

                eventInputs =
                    inputs.eventInputs
            in
            { inputs | eventInputs = { eventInputs | occurrences = eventInputs.occurrences ++ occurrenceInputs } }


updateEventInputs : Locations -> EventMsg -> EventInput -> EventInput
updateEventInputs locations msg event =
    let
        setInput new input =
            updateInput (\_ -> new) input
    in
    case msg of
        InputName newName ->
            { event | title = setInput newName event.title }

        InputTeaser newTeaser ->
            { event | teaser = setInput newTeaser event.teaser }

        InputDescription newDescription ->
            { event | description = setInput newDescription event.description }

        InputOccurrence index occurrenceMsg ->
            let
                updateOccurrence : (OccurrenceInput -> OccurrenceInput) -> EventInput
                updateOccurrence updateMapping =
                    { event
                        | occurrences =
                            List.updateAt index
                                updateMapping
                                event.occurrences
                    }
            in
            case occurrenceMsg of
                InputDuration newDuration ->
                    updateOccurrence (\occurrence -> { occurrence | duration = setInput newDuration occurrence.duration })

                InputStartDate newDate ->
                    updateOccurrence
                        (\occurrence ->
                            let
                                newStart oldStart =
                                    { oldStart | date = newDate }
                            in
                            { occurrence | start = updateInput newStart occurrence.start }
                        )

                InputStartTime newTime ->
                    updateOccurrence
                        (\occurrence ->
                            let
                                newStart oldStart =
                                    { oldStart | time = newTime }
                            in
                            { occurrence | start = updateInput newStart occurrence.start }
                        )

                InputLocationId newId ->
                    updateOccurrence
                        (\occurrence ->
                            { occurrence | locationId = setInput newId occurrence.locationId }
                        )

                InputClickedDelete ->
                    { event | occurrences = List.removeAt index event.occurrences }

        AddOccurrence ->
            let
                newOccurrences =
                    [ { start = Utils.buildInput { date = "", time = "" } Utils.dateTimeValidator
                      , duration = Utils.buildInput "" durationValidator
                      , locationId = Utils.buildInput "" (locationIdValidator locations)
                      }
                    ]
                        ++ event.occurrences
            in
            { event | occurrences = newOccurrences }


updateBatchAdd : Locations -> BatchAddMsg -> BatchAddModel -> BatchAddModel
updateBatchAdd locations msg model =
    case msg of
        BatchAddInputMsg batchMsg ->
            { model | inputs = updateBatchAddInput batchMsg model.inputs }

        BatchMultiselectCalendarMsg calendarMsg ->
            { model | dates = MultiselectCalendar.update calendarMsg model.dates }


updateBatchAddInput : BatchAddInputMsg -> BatchOccurrenceInput -> BatchOccurrenceInput
updateBatchAddInput msg input =
    let
        setInput new inp =
            updateInput (\_ -> new) inp
    in
    case msg of
        BatchInputStartTime newStartTime ->
            { input | start = setInput newStartTime input.start }

        BatchInputDuration newDuration ->
            { input | duration = setInput newDuration input.duration }

        BatchInputLocationId newLocationId ->
            { input | locationId = setInput newLocationId input.locationId }


view : Model -> List (Html Msg)
view model =
    case model of
        Valid data ->
            viewModelData data

        Invalid rawId ->
            viewInvalid rawId


viewModelData : ModelData -> List (Html Msg)
viewModelData model =
    [ Utils.breadcrumbs [ Routes.Overview ] (Routes.EditEvent <| IdDict.encodeIdForUrl model.eventId) ]
        ++ (List.map (Html.map Input) <| viewEditEvent model.locations model.inputs)
        ++ [ Utils.bottomToolbar
                [ div
                    [ css
                        [ Css.displayFlex
                        , Css.flexDirection row
                        , Css.justifyContent Css.spaceBetween
                        ]
                    ]
                    [ let
                        options =
                            { enabledness =
                                if changedAndValid model then
                                    Utils.Enabled

                                else
                                    Utils.Disabled
                            }
                      in
                      Utils.buttonWithOptions options "Speichern" ClickedSave
                    , Utils.button "Löschen" ClickedDelete
                    ]
                ]
           ]


viewEditEvent : Locations -> InputModel -> List (Html InputMsg)
viewEditEvent locations inputs =
    let
        occurrences =
            List.indexedMap
                (\index occurrence ->
                    li [] [ viewEditOccurrence locations index occurrence |> Html.map InputEvent ]
                )
                inputs.eventInputs.occurrences

        controls =
            [ div
                [ css [ Css.padding (em 1) ] ]
                [ details
                    []
                    [ summary [ css [ Css.cursor Css.pointer ] ] [ text "Neue Termine hinzufügen" ]
                    , div [ css [ Css.marginTop (em 1) ] ] (viewBatchAdd locations inputs.batchAdd)
                    ]
                ]
            ]
    in
    [ fields
        [ viewInputText "Titel" inputs.eventInputs.title (InputEvent << InputName)
        , viewInputText "Teaser" inputs.eventInputs.teaser (InputEvent << InputTeaser)
        , viewTextArea "Beschreibung" inputs.eventInputs.description (InputEvent << InputDescription)
        ]
    , h2 [] [ text "Termine" ]
    , ol [ css [ spreadListItemStyle ] ]
        ([ div
            [ css
                [ Css.marginBottom (em 1)
                , Css.position Css.sticky
                , Css.top zero
                , Css.zIndex (int 1)
                , Css.border3 (px 1) Css.solid (Css.rgb 0 0 0)
                , Css.backgroundColor (Css.hsla 0 0 1 0.9)
                , Css.maxHeight (vh 100)
                , Css.overflow Css.auto
                ]
            ]
            controls
         ]
            ++ occurrences
        )
    ]


viewBatchAdd : Locations -> BatchAddModel -> List (Html InputMsg)
viewBatchAdd locations input =
    let
        titleStyle =
            Css.batch
                [ Css.margin zero
                , Css.marginBottom (em 0.7)
                ]
    in
    [ h3 [ css [ titleStyle ] ] [ text "Daten" ]
    , MultiselectCalendar.view input.dates |> Html.map (InputBatchAdd << BatchMultiselectCalendarMsg)
    , h3 [ css [ titleStyle, Css.marginTop (em 1) ] ] [ text "Einstellungen" ]
    , div [ css [ editOccurrenceStyle ] ]
        [ viewTimeInput "Beginn"
            input.inputs.start
            (InputBatchAdd << BatchAddInputMsg << BatchInputStartTime)
        , viewInputNumber "Dauer (in Minuten)" input.inputs.duration (InputBatchAdd << BatchAddInputMsg << BatchInputDuration)
        , let
            options =
                IdDict.map (\id location -> { name = location.name, value = IdDict.encodeIdForUrl id }) locations
          in
          div []
            [ Utils.viewSelection "Ort" input.inputs.locationId options (InputBatchAdd << BatchAddInputMsg << BatchInputLocationId)
            ]
        ]
    , Utils.button "Hinzufügen" BatchAddRequested
    ]


changedAndValid : ModelData -> Bool
changedAndValid model =
    eventFromInputs model.locations model.inputs.eventInputs
        |> Maybe.map (\newEvent -> newEvent /= model.event)
        |> Maybe.withDefault False


spreadListItemStyle : Css.Style
spreadListItemStyle =
    Css.batch
        [ Css.children
            [ Css.typeSelector "li"
                [ Css.adjacentSiblings
                    [ Css.typeSelector
                        "li"
                        [ Css.marginTop (em 1)
                        ]
                    ]
                ]
            ]
        ]


editOccurrenceStyle : Css.Style
editOccurrenceStyle =
    Css.batch
        [ Css.displayFlex
        , Css.flexDirection row
        , Css.alignItems flexStart
        , Css.children
            [ Css.everything
                [ Css.adjacentSiblings
                    [ Css.everything
                        [ Css.marginLeft (em 1)
                        ]
                    ]
                , Css.paddingTop zero
                , Css.paddingBottom zero
                ]
            ]
        ]


viewEditOccurrence : Locations -> Int -> OccurrenceInput -> Html EventMsg
viewEditOccurrence locations index occurrence =
    let
        occMsg : OccurrenceMsg -> EventMsg
        occMsg subMsg =
            InputOccurrence index subMsg
    in
    div [ css [ editOccurrenceStyle ] ]
        [ viewDateTimeInput "Beginn"
            occurrence.start
            { dateChanged = occMsg << InputStartDate
            , timeChanged = occMsg << InputStartTime
            }
        , viewInputNumber "Dauer (in Minuten)" occurrence.duration (occMsg << InputDuration)
        , let
            options =
                IdDict.map (\id location -> { name = location.name, value = IdDict.encodeIdForUrl id }) locations
          in
          div []
            [ Utils.viewSelection "Ort" occurrence.locationId options (occMsg << InputLocationId)
            ]
        , Utils.button "Löschen" (occMsg InputClickedDelete)
        ]


viewInvalid : String -> List (Html Msg)
viewInvalid rawId =
    [ text "Dieses Event scheint nicht zu existieren." ]
