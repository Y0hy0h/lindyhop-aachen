module Pages.EditEvent exposing
    ( EventInput
    , InputMsg
    , LoadModel
    , LoadMsg
    , Model
    , Msg
    , eventFromInputs
    , fromEvents
    , init
    , update
    , updateInputs
    , updateLoad
    , view
    , viewEditEvent
    )

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
    { eventId : Id Event
    , event : Event
    , inputs : EventInput
    , locations : Locations
    }


type alias EventInput =
    { name : In String
    , teaser : In String
    , description : In String
    , occurrences : List OccurrenceInput
    }


type alias OccurrenceInput =
    { start : Input { date : String, time : String } Naive.DateTime
    , duration : In Duration
    , locationId : In (Id Location)
    }


eventFromInputs : Locations -> EventInput -> Maybe Event
eventFromInputs locs inputs =
    let
        maybeOccurrences =
            Maybe.combine (List.map (occurrenceFromInput locs) inputs.occurrences)
    in
    Maybe.map4 Event
        (extract inputs.name)
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
    { name = inputString event.name
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

                    inputs =
                        inputsFromEvent locations event
                in
                Model id event inputs locations
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
    | DeleteFinished (Result Http.Error Event)


type InputMsg
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


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
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
                    case eventFromInputs model.locations model.inputs of
                        Just event ->
                            Events.updateEvent model.eventId event SaveFinished

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


updateInputs : Locations -> InputMsg -> EventInput -> EventInput
updateInputs locations msg event =
    let
        setInput new input =
            updateInput (\_ -> new) input
    in
    case msg of
        InputName newName ->
            { event | name = setInput newName event.name }

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
                    event.occurrences
                        ++ [ { start = Utils.buildInput { date = "", time = "" } Utils.dateTimeValidator
                             , duration = Utils.buildInput "" durationValidator
                             , locationId = Utils.buildInput "" (locationIdValidator locations)
                             }
                           ]
            in
            { event | occurrences = newOccurrences }


updateEvent : Model -> (EventInput -> EventInput) -> Model
updateEvent model eventUpdater =
    let
        newEvent =
            eventUpdater model.inputs
    in
    { model | inputs = newEvent }


updateOccurrences : Model -> (List OccurrenceInput -> List OccurrenceInput) -> Model
updateOccurrences model occurrencesUpdater =
    let
        newOccurrences =
            occurrencesUpdater model.inputs.occurrences

        inputs =
            model.inputs

        newInputs =
            { inputs | occurrences = newOccurrences }
    in
    { model | inputs = newInputs }


view : Model -> List (Html Msg)
view model =
    [ Utils.breadcrumbs [ Routes.Overview ] (Routes.EditEvent <| IdDict.encodeIdForUrl model.eventId) ]
        ++ (List.map (Html.map Input) <| viewEditEvent model.locations model.inputs)
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


viewEditEvent : Locations -> EventInput -> List (Html InputMsg)
viewEditEvent locations inputs =
    [ fields
        [ viewInputText "Titel" inputs.name InputName
        , viewInputText "Teaser" inputs.teaser InputTeaser
        , viewTextArea "Beschreibung" inputs.description InputDescription
        ]
    , h2 [] [ text "Termine" ]
    , ol [ css [ spreadListItemStyle ] ]
        (List.indexedMap
            (\index occurrence ->
                li [] [ viewEditOccurrence locations index occurrence ]
            )
            inputs.occurrences
            ++ [ Utils.button "Neuer Termin" AddOccurrence ]
        )
    ]


changed : Model -> Bool
changed model =
    eventFromInputs model.locations model.inputs
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


viewEditOccurrence : Locations -> Int -> OccurrenceInput -> Html InputMsg
viewEditOccurrence locations index occurrence =
    let
        occMsg : OccurrenceMsg -> InputMsg
        occMsg subMsg =
            InputOccurrence index subMsg

        occurrenceStyle =
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
    in
    div [ css [ occurrenceStyle ] ]
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
            ([ Utils.viewSelection "Ort" occurrence.locationId options (occMsg << InputLocationId)
             ]
                ++ (case extract occurrence.locationId of
                        Just id ->
                            [ a [ href <| Routes.toRelativeUrl <| Routes.EditLocation (IdDict.encodeIdForUrl id) ] [ text "Bearbeiten" ] ]

                        Nothing ->
                            []
                   )
            )
        , Utils.button "Löschen" (occMsg InputClickedDelete)
        ]
