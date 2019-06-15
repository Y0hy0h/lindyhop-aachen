module Utils.MultiselectCalendar exposing (Model, Msg, init, selected, update, view)

import Date exposing (Date)
import Html exposing (Html, a, button, div, form, input, li, table, tbody, td, text, th, thead, tr, ul)
import Html.Attributes exposing (class, classList, type_, value)
import Html.Events exposing (onClick, onInput, onSubmit)
import Json.Encode as Encode
import List.Extra as List
import Parser exposing ((|.), (|=))
import Task
import Time
import Utils.MultiselectCalendar.Calendar as Calendar



-- MODEL


type Model
    = Loading
    | Loaded DatesModel


type alias DatesModel =
    { today : Date
    , month : Date
    , selected : List Date
    , dateInput : String
    }


init : List String -> ( Model, Cmd Msg )
init rawDates =
    let
        dates =
            List.filterMap parseDate rawDates
    in
    ( Loading, Task.perform (SetToday dates) Date.today )


selected : Model -> List Date
selected model =
    case model of
        Loading ->
            []

        Loaded dates ->
            dates.selected



-- UPDATE


type Msg
    = SetToday (List Date) Date
    | DatesMsg DatesMsg


update : Msg -> Model -> Model
update msg model =
    case msg of
        SetToday prefilledDates todayDate ->
            Loaded
                { today = todayDate
                , month = monthFromDate todayDate
                , selected = prefilledDates
                , dateInput = ""
                }

        DatesMsg datesMsg ->
            case model of
                Loaded datesModel ->
                    updateDates datesMsg datesModel
                        |> Loaded

                _ ->
                    model


type DatesMsg
    = CombinedActionMsg DateAction (Maybe MonthAction)
    | MonthActionMsg MonthAction
    | GoToCurrentMonth
    | GoToDate Date
    | DateInputChanged String
    | DateInputSubmitted


type DateAction
    = Add Date
    | Remove Date


type MonthAction
    = PreviousMonth
    | NextMonth


updateDates : DatesMsg -> DatesModel -> DatesModel
updateDates msg model =
    case msg of
        CombinedActionMsg dateAction maybeMonthAction ->
            let
                dateModel =
                    updateSelection model dateAction
            in
            updateMonth maybeMonthAction dateModel

        MonthActionMsg monthAction ->
            updateMonth (Just monthAction) model

        GoToCurrentMonth ->
            { model | month = monthFromDate model.today }

        GoToDate date ->
            { model | month = monthFromDate date }

        DateInputChanged newDate ->
            { model | dateInput = newDate }

        DateInputSubmitted ->
            case parseDate model.dateInput of
                Just date ->
                    let
                        newModel =
                            updateSelection model (Add date)
                    in
                    { newModel | dateInput = "" }

                Nothing ->
                    model


monthFromDate : Date -> Date
monthFromDate date =
    Date.floor Date.Month date


parseDate : String -> Maybe Date
parseDate rawDate =
    let
        paddedInt =
            Parser.succeed identity
                |. Parser.chompWhile (\c -> c == '0')
                |= Parser.int

        month =
            Parser.succeed Date.numberToMonth
                |= paddedInt

        date =
            Parser.succeed Date.fromCalendarDate
                |= paddedInt
                |. Parser.symbol "."
                |= month
                |. Parser.symbol "."
                |= paddedInt
    in
    case Date.fromIsoString rawDate of
        Ok d ->
            Just d

        Err _ ->
            Parser.run date rawDate |> Result.toMaybe


updateSelection : DatesModel -> DateAction -> DatesModel
updateSelection model action =
    let
        newModel =
            case action of
                Add newDate ->
                    { model | selected = model.selected ++ [ newDate ] }

                Remove oldDate ->
                    let
                        newSelected =
                            List.remove oldDate model.selected
                    in
                    { model | selected = newSelected }

        sortedModel =
            { newModel | selected = List.sortBy Date.toIsoString newModel.selected }
    in
    sortedModel


updateMonth : Maybe MonthAction -> DatesModel -> DatesModel
updateMonth maybeMonthAction model =
    let
        moveMonth step month =
            Date.add Date.Months step month
    in
    case maybeMonthAction of
        Just PreviousMonth ->
            { model | month = moveMonth -1 model.month }

        Just NextMonth ->
            { model | month = moveMonth 1 model.month }

        Nothing ->
            model



-- VIEW


view : Model -> Html Msg
view model =
    case model of
        Loading ->
            text "Loading"

        Loaded datesModel ->
            viewDates datesModel
                |> Html.map DatesMsg


viewDates : DatesModel -> Html DatesMsg
viewDates model =
    div [ class "calendar" ]
        [ div [ class "container" ]
            [ div [ class "calendar-header" ]
                [ button
                    [ class "today"
                    , onClick GoToCurrentMonth
                    ]
                    [ text "This month" ]
                , text (Date.format "MMMM y" model.month)
                ]
            , button
                [ class "previous-month"
                , onClick (MonthActionMsg PreviousMonth)
                ]
                [ text "Previous month" ]
            , viewCalendar model
            , button
                [ class "next-month"
                , onClick (MonthActionMsg NextMonth)
                ]
                [ text "Next month" ]
            ]
        , viewDatesList model.dateInput model.selected
        ]


button : List (Html.Attribute DatesMsg) -> List (Html DatesMsg) -> Html DatesMsg
button attributes contents =
    Html.button (attributes ++ [ type_ "button" ]) contents


viewCalendar : DatesModel -> Html DatesMsg
viewCalendar model =
    let
        calendar =
            Calendar.forMonth (Date.year model.month) (Date.month model.month)
    in
    table []
        [ thead []
            [ tr []
                [ th [] [ text "Mo" ]
                , th [] [ text "Di" ]
                , th [] [ text "Mi" ]
                , th [] [ text "Do" ]
                , th [] [ text "Fr" ]
                , th [] [ text "Sa" ]
                , th [] [ text "So" ]
                ]
            ]
        , tbody []
            (List.map
                (\week ->
                    tr []
                        (List.map
                            (\day ->
                                let
                                    date =
                                        Calendar.dateFromCalendarDate day

                                    isToday =
                                        date == model.today

                                    isSelected =
                                        List.member date model.selected
                                in
                                viewCalendarDay { today = isToday, selected = isSelected } day
                            )
                            week
                        )
                )
                calendar
            )
        ]


viewCalendarDay : { today : Bool, selected : Bool } -> Calendar.CalendarDate -> Html DatesMsg
viewCalendarDay is day =
    let
        ( dayClass, dayDate, maybeMonthAction ) =
            case day of
                Calendar.Previous date ->
                    ( "previous", date, Just PreviousMonth )

                Calendar.Current date ->
                    ( "current", date, Nothing )

                Calendar.Next date ->
                    ( "next", date, Just NextMonth )

        dateAction =
            if is.selected then
                Remove dayDate

            else
                Add dayDate
    in
    td
        [ classList
            [ ( dayClass, True )
            , ( "today", is.today )
            , ( "selected", is.selected )
            ]
        , onClick (CombinedActionMsg dateAction maybeMonthAction)
        ]
        [ text (String.fromInt <| Date.day dayDate)
        ]


viewDatesList : String -> List Date -> Html DatesMsg
viewDatesList currentInput dates =
    let
        viewDateListItem date =
            li []
                [ button [ class "goto-date", onClick (GoToDate date) ] [ text "Go to" ]
                , text (Date.format "dd.MM.yyyy" date)
                , button [ class "remove-date", onClick (CombinedActionMsg (Remove date) Nothing) ] [ text "Remove" ]
                ]
    in
    div [ class "selected-list" ]
        [ form [ onSubmit DateInputSubmitted ]
            [ input [ type_ "date", onInput DateInputChanged, value currentInput ] []
            , Html.button [ class "add-date", type_ "submit" ] [ text "Add date" ]
            ]
        , ul [] (List.map viewDateListItem dates)
        ]
