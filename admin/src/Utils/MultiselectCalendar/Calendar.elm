module Utils.MultiselectCalendar.Calendar exposing (CalendarDate(..), CalendarMonth, dateFromCalendarDate, forMonth)

import Date exposing (Date)
import Time


type alias CalendarMonth =
    List (List CalendarDate)


type CalendarDate
    = Current Date
    | Previous Date
    | Next Date


dateFromCalendarDate : CalendarDate -> Date
dateFromCalendarDate calendarDate =
    case calendarDate of
        Current date ->
            date

        Previous date ->
            date

        Next date ->
            date


forMonth : Int -> Time.Month -> CalendarMonth
forMonth year month =
    let
        days =
            calendarMonthDays year month
    in
    List.foldl
        (\day ( week, result ) ->
            if List.length week < 7 then
                ( week ++ [ day ], result )

            else
                ( [ day ], result ++ [ week ] )
        )
        ( [], [] )
        days
        |> (\( week, result ) -> result ++ [ week ])


calendarMonthDays : Int -> Time.Month -> List CalendarDate
calendarMonthDays year month =
    let
        beginningOfMonth =
            Date.fromCalendarDate year month 1

        start =
            firstOfSameWeek beginningOfMonth
                |> Date.add Date.Weeks -1

        untilEndOfMonth =
            Date.add Date.Months 1 beginningOfMonth

        until =
            lastOfSameWeek start
                |> Date.add Date.Weeks 8

        previous =
            Date.range Date.Day 1 start beginningOfMonth
                |> List.map Previous

        current =
            Date.range Date.Day 1 beginningOfMonth untilEndOfMonth
                |> List.map Current

        next =
            Date.range Date.Day 1 untilEndOfMonth until
                |> List.map Next
    in
    previous ++ current ++ next


firstOfSameWeek : Date -> Date
firstOfSameWeek date =
    Date.floor Date.Week date


lastOfSameWeek : Date -> Date
lastOfSameWeek date =
    Date.ceiling Date.Week date
