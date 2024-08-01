#!/bin/bash

source ./test_utils.sh

# GitApp
echo -e "\nTesting Project Management Policies..."
validate "projects" "policies.cedar" "projects.cedarschema"
authorize "projects" "policies.cedar" "entities.json"

exit "$any_failed"