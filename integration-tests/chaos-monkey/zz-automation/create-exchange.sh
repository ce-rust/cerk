#!/bin/bash

rabbitmqadmin -V / declare exchange type=fanout name=input durable=false
