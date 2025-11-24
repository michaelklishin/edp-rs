%% Copyright (C) 2025-2026 Michael S. Klishin and Contributors
%%
%% Licensed under the Apache License, Version 2.0 (the "License");
%% you may not use this file except in compliance with the License.
%% You may obtain a copy of the License at
%%
%% http://www.apache.org/licenses/LICENSE-2.0
%%
%% Unless required by applicable law or agreed to in writing, software
%% distributed under the License is distributed on an "AS IS" BASIS,
%% WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
%% See the License for the specific language governing permissions and
%% limitations under the License.

-module(test_node).
-export([start/2, test_function/0, echo/1, test_tuple/0, add/2, multiply/2,
         list_length/1, reverse_list/1, make_list/1, get_node_name/0,
         return_error/0, atom_to_string/1]).

start(NodeName, Hostname) when is_atom(NodeName), is_atom(Hostname) ->
    FullName = list_to_atom(atom_to_list(NodeName) ++ "@" ++ atom_to_list(Hostname)),
    net_kernel:start([FullName, shortnames]),
    erlang:set_cookie('monster'),
    receive
        stop -> ok
    end.

test_function() ->
    {ok, "Hello from Erlang!"}.

echo(Term) ->
    Term.

test_tuple() ->
    {ok, hello, world}.

add(A, B) when is_integer(A), is_integer(B) ->
    A + B.

multiply(A, B) when is_integer(A), is_integer(B) ->
    A * B.

list_length(List) when is_list(List) ->
    length(List).

reverse_list(List) when is_list(List) ->
    lists:reverse(List).

make_list(N) when is_integer(N), N >= 0 ->
    lists:seq(1, N).

get_node_name() ->
    node().

return_error() ->
    {error, intentional_error}.

atom_to_string(Atom) when is_atom(Atom) ->
    atom_to_list(Atom).
