module Utils.SimpleTime exposing (SimpleTime, fromPosix, hour, minute)

import Time


type SimpleTime
    = SimpleTime Int Int


fromPosix : Time.Zone -> Time.Posix -> SimpleTime
fromPosix zone time =
    let
        hour_ =
            Time.toHour zone time

        minute_ =
            Time.toMinute zone time
    in
    SimpleTime hour_ minute_


hour : SimpleTime -> Int
hour (SimpleTime hour_ _) =
    hour_


minute : SimpleTime -> Int
minute (SimpleTime _ minute_) =
    minute_
