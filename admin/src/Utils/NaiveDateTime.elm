module Utils.NaiveDateTime exposing
    ( Date
    , DateTime
    , Duration
    , Time
    , asMinutes
    , build
    , buildDate
    , buildTime
    , dateParser
    , dateTimeFuzzer
    , dateTimeParser
    , day
    , decodeDateTime
    , decodeMinutes
    , encodeAsMinutes
    , encodeDateAsString
    , encodeDateTime
    , encodeTimeAsString
    , hour
    , minute
    , minutes
    , month
    , monthNumeric
    , setDate
    , setTime
    , timeParser
    , with
    , year
    )

import Fuzz
import Json.Decode as Decode
import Json.Encode as Encode
import Parser exposing ((|.), (|=), Parser, end, symbol)
import Time
import Utils.Format exposing (padInt)


type DateTime
    = DateTime Date Time


type Date
    = Date { year : Int, month : Time.Month, day : Int }


type Time
    = Time { hour : Int, minute : Int }


type Duration
    = Minutes Int


asMinutes : Duration -> Int
asMinutes (Minutes n) =
    n



-- Create


type BuildError
    = DateError BuildDateError
    | TimeError BuildTimeError
    | DateAndTimeError BuildDateError BuildTimeError


with : Date -> Time -> DateTime
with date time =
    DateTime date time


build : { year : Int, month : Int, day : Int, hour : Int, minute : Int } -> Result BuildError DateTime
build v =
    let
        date =
            buildDate { year = v.year, month = v.month, day = v.day }

        time =
            buildTime { hour = v.hour, minute = v.minute }
    in
    case ( date, time ) of
        ( Ok d, Ok t ) ->
            Ok <| DateTime d t

        ( Err dateError, Ok _ ) ->
            Err <| DateError dateError

        ( Ok _, Err timeError ) ->
            Err <| TimeError timeError

        ( Err dateError, Err timeError ) ->
            Err <| DateAndTimeError dateError timeError


type BuildDateError
    = InvalidYear
    | InvalidMonth
    | InvalidYearAndMonth
    | InvalidDay


buildDate : { year : Int, month : Int, day : Int } -> Result BuildDateError Date
buildDate v =
    let
        yearResult =
            if v.year > 0 then
                Just v.year

            else
                Nothing

        monthResult =
            monthFromNumber v.month
    in
    case ( yearResult, monthResult ) of
        ( Just y, Just m ) ->
            if v.day > 0 && v.day <= daysInMonth y m then
                Ok <| Date { year = y, month = m, day = v.day }

            else
                Err InvalidDay

        ( Nothing, Just _ ) ->
            Err InvalidYear

        ( Just _, Nothing ) ->
            Err InvalidMonth

        ( Nothing, Nothing ) ->
            Err InvalidYearAndMonth


type BuildTimeError
    = InvalidHour
    | InvalidMinute
    | InvalidHourAndMinute


buildTime : { hour : Int, minute : Int } -> Result BuildTimeError Time
buildTime v =
    let
        hourResult =
            if v.hour >= 0 && v.hour < 24 then
                Just v.hour

            else
                Nothing

        minuteResult =
            if v.minute >= 0 && v.minute < 60 then
                Just v.minute

            else
                Nothing
    in
    case ( hourResult, minuteResult ) of
        ( Just h, Just m ) ->
            Ok <| Time { hour = h, minute = m }

        ( Just _, Nothing ) ->
            Err InvalidHour

        ( Nothing, Just _ ) ->
            Err InvalidMinute

        ( Nothing, Nothing ) ->
            Err InvalidHourAndMinute


minutes : Int -> Maybe Duration
minutes raw =
    if raw >= 0 then
        Just <| Minutes raw

    else
        Nothing



-- Query


year : DateTime -> Int
year (DateTime (Date date) _) =
    date.year


month : DateTime -> Time.Month
month (DateTime (Date date) _) =
    date.month


monthNumeric : DateTime -> Int
monthNumeric dateTime =
    month dateTime
        |> numberFromMonth


day : DateTime -> Int
day (DateTime (Date date) _) =
    date.day


hour : DateTime -> Int
hour (DateTime _ (Time time)) =
    time.hour


minute : DateTime -> Int
minute (DateTime _ (Time time)) =
    time.minute



-- Manipulate


setDate : Date -> DateTime -> DateTime
setDate newDate (DateTime _ time) =
    DateTime newDate time


setTime : Time -> DateTime -> DateTime
setTime newTime (DateTime date _) =
    DateTime date newTime



-- JSON


decodeDateTime : Decode.Decoder DateTime
decodeDateTime =
    Decode.string
        |> Decode.andThen
            (\raw ->
                case Parser.run dateTimeParser raw of
                    Ok dateTime ->
                        Decode.succeed dateTime

                    Err error ->
                        let
                            stringFromDeadEnd deadEnd =
                                case deadEnd.problem of
                                    Parser.Problem prob ->
                                        prob

                                    Parser.ExpectingSymbol symbol ->
                                        "Expected symbol '"
                                            ++ symbol
                                            ++ "' at row "
                                            ++ String.fromInt deadEnd.row
                                            ++ ", column "
                                            ++ String.fromInt deadEnd.col
                                            ++ "."

                                    _ ->
                                        "The value could not be parsed."
                        in
                        Decode.fail (List.map stringFromDeadEnd error |> String.join "\n")
            )


decodeMinutes : Decode.Decoder Duration
decodeMinutes =
    Decode.int
        |> Decode.andThen
            (\raw ->
                minutes raw
                    |> Maybe.map Decode.succeed
                    |> Maybe.withDefault (Decode.fail "Invalid duration in minutes.")
            )


encodeDateTime : DateTime -> Encode.Value
encodeDateTime dateTime =
    Encode.string
        (encodeDateAsString dateTime ++ "T" ++ encodeTimeAsString dateTime)


encodeDateAsString : DateTime -> String
encodeDateAsString dateTime =
    let
        y =
            String.fromInt <| year dateTime

        m =
            padInt <| monthNumeric dateTime

        d =
            padInt <| day dateTime
    in
    y ++ "-" ++ m ++ "-" ++ d


encodeTimeAsString : DateTime -> String
encodeTimeAsString dateTime =
    let
        h =
            padInt <| hour dateTime

        min =
            padInt <| minute dateTime
    in
    h ++ ":" ++ min ++ ":00"


dateTimeParser : Parser DateTime
dateTimeParser =
    Parser.succeed
        (\date time ->
            DateTime date time
        )
        |= dateParser
        |. symbol "T"
        |= timeParser


encodeAsMinutes : Duration -> Encode.Value
encodeAsMinutes (Minutes n) =
    Encode.int n


dateParser : Parser Date
dateParser =
    Parser.succeed
        (\y m d -> { year = y, month = m, day = d })
        |= Parser.int
        |. symbol "-"
        |= paddedInt
        |. symbol "-"
        |= paddedInt
        |> Parser.andThen
            (\values ->
                case buildDate values of
                    Ok date ->
                        Parser.succeed date

                    Err error ->
                        Parser.problem <| stringFromDateError error
            )


timeParser : Parser Time
timeParser =
    Parser.succeed
        (\h m -> { hour = h, minute = m })
        |= paddedInt
        |. symbol ":"
        |= paddedInt
        |> Parser.andThen
            (\values ->
                case buildTime values of
                    Ok time ->
                        Parser.succeed time

                    Err error ->
                        Parser.problem <| stringFromTimeError error
            )


paddedInt : Parser Int
paddedInt =
    Parser.oneOf
        [ Parser.succeed identity
            |. symbol "0"
            |= Parser.int
        , Parser.int
        ]


stringFromDateError : BuildDateError -> String
stringFromDateError dateError =
    case dateError of
        InvalidYear ->
            "The year is invalid."

        InvalidMonth ->
            "The month is invalid."

        InvalidYearAndMonth ->
            "The year and month are invalid."

        InvalidDay ->
            "The day is not valid for that month and year."


stringFromTimeError : BuildTimeError -> String
stringFromTimeError timeError =
    case timeError of
        InvalidHour ->
            "The hour is invalid."

        InvalidMinute ->
            "The minute is invalid."

        InvalidHourAndMinute ->
            "The hour and minute are invalid."


stringFromBuildError : BuildError -> String
stringFromBuildError buildError =
    case buildError of
        DateError dateError ->
            stringFromDateError dateError

        TimeError timeError ->
            stringFromTimeError timeError

        DateAndTimeError dateError timeError ->
            stringFromDateError dateError ++ " " ++ stringFromTimeError timeError



-- Testing


dateTimeFuzzer : Fuzz.Fuzzer DateTime
dateTimeFuzzer =
    Fuzz.map5
        (\y m d h minute_ ->
            let
                validMonth =
                    monthFromNumber m |> Maybe.withDefault Time.Jan

                validDay =
                    min d (daysInMonth y validMonth)
            in
            DateTime
                (Date { year = y, month = validMonth, day = validDay })
                (Time { hour = h, minute = minute_ })
        )
        (Fuzz.intRange 1 3000)
        (Fuzz.intRange 1 12)
        (Fuzz.intRange 1 31)
        (Fuzz.intRange 0 23)
        (Fuzz.intRange 0 59)



-- Utils


monthFromNumber : Int -> Maybe Time.Month
monthFromNumber rawMonth =
    case rawMonth of
        1 ->
            Just Time.Jan

        2 ->
            Just Time.Feb

        3 ->
            Just Time.Mar

        4 ->
            Just Time.Apr

        5 ->
            Just Time.May

        6 ->
            Just Time.Jun

        7 ->
            Just Time.Jul

        8 ->
            Just Time.Aug

        9 ->
            Just Time.Sep

        10 ->
            Just Time.Oct

        11 ->
            Just Time.Nov

        12 ->
            Just Time.Dec

        _ ->
            Nothing


numberFromMonth : Time.Month -> Int
numberFromMonth m =
    case m of
        Time.Jan ->
            1

        Time.Feb ->
            2

        Time.Mar ->
            3

        Time.Apr ->
            4

        Time.May ->
            5

        Time.Jun ->
            6

        Time.Jul ->
            7

        Time.Aug ->
            8

        Time.Sep ->
            9

        Time.Oct ->
            10

        Time.Nov ->
            11

        Time.Dec ->
            12



{-
   The following functions are taken from https://github.com/justinmimbs/date/blob/3.1.2/src/Date.elm,
   which is published under the following license. Namespacing was added where necessary.

   BSD 3-Clause

   Copyright (c) 2018, Justin Mimbs. All rights reserved.

   Redistribution and use in source and binary forms, with or without
   modification, are permitted provided that the following conditions are met:

   1. Redistributions of source code must retain the above copyright notice, this
      list of conditions and the following disclaimer.
   2. Redistributions in binary form must reproduce the above copyright notice,
      this list of conditions and the following disclaimer in the documentation
      and/or other materials provided with the distribution.
   3. Neither the name of the copyright holder nor the names of its contributors
      may be used to endorse or promote products derived from this software
      without specific prior written permission.

   THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
   ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
   WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
   DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
   FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
   DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
   SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
   CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
   OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
   OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
-}


daysInMonth : Int -> Time.Month -> Int
daysInMonth y m =
    case m of
        Time.Jan ->
            31

        Time.Feb ->
            if isLeapYear y then
                29

            else
                28

        Time.Mar ->
            31

        Time.Apr ->
            30

        Time.May ->
            31

        Time.Jun ->
            30

        Time.Jul ->
            31

        Time.Aug ->
            31

        Time.Sep ->
            30

        Time.Oct ->
            31

        Time.Nov ->
            30

        Time.Dec ->
            31


isLeapYear : Int -> Bool
isLeapYear y =
    modBy 4 y == 0 && modBy 100 y /= 0 || modBy 400 y == 0
