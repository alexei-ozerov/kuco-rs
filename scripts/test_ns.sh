#!/usr/bin/env bash

NS_LIST='test-01 test-02 test-03 test-04'

echo "Creating test namespaces."
for NS in $NS_LIST; do
    kubectl create ns $NS
done

echo "Waiting for 5 seconds."
sleep 5s

echo "Deleting namespaces with 5 second interval."
for NS in $NS_LIST; do 
    kubectl delete ns $NS 
    sleep 5s 
done 

echo "Test completed."

exit 0
