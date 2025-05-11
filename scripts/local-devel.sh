#!/bin/bash 

cargo build

CLUSTER_NAME=local-01

echo "Creating KinD cluster named ${CLUSTER_NAME}."

kind create cluster -n "${CLUSTER_NAME}"
