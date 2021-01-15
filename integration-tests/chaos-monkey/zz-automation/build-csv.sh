#!/bin/sh

set -e

echo "name;setup;mode;missing;start;end;brokercrash;ended" > output.csv

for p in $( ls ./output/*.log ); do
    f=$(basename -- "$p");
    name=$f
    setup=$(echo $f | sed -r 's/(.*)-(un)?reliable.*/\1/g')
    mode=$(echo $f | sed -r 's/.*-((un)?reliable).*/\1/g')
    missing=$(awk '/missing events: ([0-9]+)/{a=$0}END{print a}' $p | sed -r 's/.*missing events: ([0-9]+)/\1/g')
    start=$(cat $p | grep "end:" | sed -r 's/end: ([0-9]+)/\1/g')
    end=$(cat $p |  grep "sequence_generator_started:" | sed -r 's/sequence_generator_started: ([0-9]+)/\1/g')
    crash=$(cat $p | grep "a broker crashed during the test" | wc -l)
    ended=$(cat $p | grep "no broker restarts - ok" | wc -l)
    echo "${name};${setup};${mode};${missing};${start};${end};${crash};${ended}" >> output.csv
done
