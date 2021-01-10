#!/bin/bash

rabbitmqadmin -V / declare exchange type=fanout name=input durable=false
rabbitmqadmin -V / declare exchange type=fanout name=output durable=false
