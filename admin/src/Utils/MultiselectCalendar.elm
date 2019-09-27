module Utils.MultiselectCalendar exposing (Model, Msg, init, selected, update, view)

import Css exposing (em, flexStart, pct, px, rem, row, zero)
import Date exposing (Date)
import Html.Styled as Html exposing (Html, a, button, div, form, input, li, table, tbody, td, text, th, thead, tr, ul)
import Html.Styled.Attributes exposing (css, type_, value)
import Html.Styled.Events exposing (onClick, onInput, onSubmit)
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
            text "Lädt..."

        Loaded datesModel ->
            viewDates datesModel
                |> Html.map DatesMsg


viewDates : DatesModel -> Html DatesMsg
viewDates model =
    let
        navButtonStyle =
            Css.batch
                [ Css.display Css.block
                , Css.width (pct 100)
                ]
    in
    div
        [ css
            [ Css.property "display" "inline-grid"
            , Css.property "grid-template-columns" "auto auto"
            , Css.property "grid-gap" "1em"
            , Css.property "user-select" "none"
            ]
        ]
        [ div [ css [ Css.display Css.inlineBlock ] ]
            [ div
                [ css
                    [ Css.displayFlex
                    , Css.flexDirection Css.column
                    , Css.paddingBottom (em 0.5)
                    ]
                ]
                [ button
                    [ onClick GoToCurrentMonth
                    ]
                    [ text "Jetziger Monat" ]
                , text (formatMonth model.month)
                ]
            , button
                [ css [ navButtonStyle ]
                , onClick (MonthActionMsg PreviousMonth)
                ]
                [ text "Vorheriger Monat" ]
            , viewCalendar model
            , button
                [ css [ navButtonStyle ]
                , onClick (MonthActionMsg NextMonth)
                ]
                [ text "Nächster Monat" ]
            ]
        , viewDatesList model.dateInput model.selected
        ]


formatMonth : Date -> String
formatMonth date =
    let
        month =
            case Date.month date of
                Time.Jan ->
                    "Januar"

                Time.Feb ->
                    "Februar"

                Time.Mar ->
                    "März"

                Time.Apr ->
                    "April"

                Time.May ->
                    "Mai"

                Time.Jun ->
                    "Juni"

                Time.Jul ->
                    "Juli"

                Time.Aug ->
                    "August"

                Time.Sep ->
                    "September"

                Time.Oct ->
                    "Oktober"

                Time.Nov ->
                    "November"

                Time.Dec ->
                    "Dezember"

        year =
            Date.year date |> String.fromInt
    in
    month ++ " " ++ year


button : List (Html.Attribute DatesMsg) -> List (Html DatesMsg) -> Html DatesMsg
button attributes contents =
    Html.button
        (attributes
            ++ [ type_ "button"
               , css
                    [ Css.border3 (px 1) Css.solid (Css.rgb 0 0 0)
                    , Css.hover [ Css.property "filter" "brightness(90%)" ]
                    ]
               ]
        )
        contents


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
        outsideStyle =
            Css.color (Css.rgb 100 100 100)

        ( dayStyle, dayDate, maybeMonthAction ) =
            case day of
                Calendar.Previous date ->
                    ( [ outsideStyle ], date, Just PreviousMonth )

                Calendar.Current date ->
                    ( [], date, Nothing )

                Calendar.Next date ->
                    ( [ outsideStyle ], date, Just NextMonth )

        todayStyle =
            if is.today then
                [ Css.textDecoration Css.underline ]

            else
                []

        selectedStyle =
            if is.selected then
                [ Css.backgroundColor (Css.rgba 0 0 0 0.2) ]

            else
                []

        tdStyle =
            dayStyle
                ++ todayStyle
                ++ selectedStyle
                ++ [ Css.textAlign Css.right
                   , Css.border (em 0)
                   , Css.hover
                        [ Css.cursor Css.pointer
                        , if is.selected then
                            Css.backgroundColor (Css.rgba 0 0 0 0.3)

                          else
                            Css.backgroundColor (Css.rgba 0 0 0 0.1)
                        ]
                   ]

        dateAction =
            if is.selected then
                Remove dayDate

            else
                Add dayDate
    in
    td
        [ css tdStyle
        , onClick (CombinedActionMsg dateAction maybeMonthAction)
        ]
        [ text (String.fromInt <| Date.day dayDate)
        ]


viewDatesList : String -> List Date -> Html DatesMsg
viewDatesList currentInput dates =
    let
        viewDateListItem date =
            li
                [ css
                    [ Css.property "display" "grid"
                    , Css.property "grid-template-columns" "repeat(3, auto)"
                    , Css.property "grid-gap" "0.5em"
                    , Css.alignItems Css.center
                    ]
                ]
                [ text (Date.format "dd.MM.yyyy" date)
                , button
                    [ onClick (GoToDate date)
                    ]
                    [ text "Zeigen" ]
                , button
                    [ onClick (CombinedActionMsg (Remove date) Nothing)
                    ]
                    [ text "Löschen" ]
                ]
    in
    div []
        [ form
            [ css
                [ Css.displayFlex
                , Css.flexDirection Css.row
                ]
            , onSubmit DateInputSubmitted
            ]
            [ input [ type_ "date", onInput DateInputChanged, value currentInput ] []
            , Html.button
                [ type_ "submit"
                ]
                [ text "Datum hinzufügen" ]
            ]
        , ul
            [ css
                [ Css.height (em 18)
                , Css.overflow Css.auto
                , Css.padding zero
                ]
            ]
            (List.map viewDateListItem dates)
        ]
