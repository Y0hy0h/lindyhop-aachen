module Events exposing
    ( EventsTree
    , OccurrenceList
    , dateTreeFromOccurrenceList
    , occurrenceListFromEventsTree
    )

import Date exposing (Date)
import Dict exposing (Dict)
import Events.Event as Event exposing (Event, FullEvent)
import Events.Occurrence as Occurrence exposing (Occurrence, OccurrenceTime)
import Time


type alias EventsTree =
    List FullEvent


type alias OccurrenceList =
    List ( Occurrence, Event )


occurrenceListFromEventsTree : EventsTree -> OccurrenceList
occurrenceListFromEventsTree fullEvents =
    let
        sortByOccurrences : List FullEvent -> List ( Occurrence, Event )
        sortByOccurrences fullEvents_ =
            List.concatMap
                (\fullEvent ->
                    let
                        event =
                            fullEvent.event
                    in
                    List.map
                        (\occurrence -> ( occurrence, event ))
                        fullEvent.occurrences
                )
                fullEvents_
    in
    sortByOccurrences fullEvents


type alias DateTree =
    List ( Date, List ( OccurrenceTime, Event ) )


type alias RataDie =
    Int


dateTreeFromOccurrenceList : Time.Zone -> OccurrenceList -> DateTree
dateTreeFromOccurrenceList zone occurrences =
    List.foldl
        (\( occurrence, event ) dict ->
            let
                ( date, occurrenceTime ) =
                    Occurrence.splitOccurrence zone occurrence

                rataDie =
                    Date.toRataDie date

                update maybeKey =
                    Maybe.withDefault [] maybeKey
                        |> List.append [ ( occurrenceTime, event ) ]
                        |> Just
            in
            Dict.update rataDie update dict
        )
        Dict.empty
        occurrences
        |> Dict.toList
        |> List.map (Tuple.mapFirst Date.fromRataDie)
