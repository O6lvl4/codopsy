-module(sample).
-export([process/1, empty/0]).

%% TODO: add proper error handling

process(Data) ->
    case Data of
        _ -> ok;
        {error, Reason} -> {error, Reason};
        {ok, Value} -> Value
    end.

empty() -> ok.

risky(X) ->
    exit(X).
