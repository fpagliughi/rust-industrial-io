#!/bin/bash
#
# load_dummy.sh
#
# Loads the IIO dummy device. 
# This is useful for application testing and unit testing of the library.
#

IIO_DEV_NAME=dummydev

# ----- Must have root privleges to run this script -----

if (( $EUID != 0 )); then
   echo "This script must be run as root"
   exit 1
fi

# ----- Load the kernel module(s) -----

DUMMY_LOADED=$(lsmod | grep ^iio_dummy)

if [ -n "${DUMMY_LOADED}" ]; then
    printf "IIO dummy module already loaded.\n"
else
    if ! modprobe iio_dummy ; then
        printf "Unable to load load the IIO dummy module.\n"
        exit 1
    fi
    printf "IIO dummy module loaded.\n"
fi

# ----- Mount the config filesystem -----

CONFIG_PATH=$(mount | awk '/^configfs/ { print $3 }')

if [ -n "$CONFIG_PATH" ]; then
    printf "Found configfs at '%s'\n" "${CONFIG_PATH}"
else
    CONFIG_PATH=/mnt/config
    mkdir -p ${CONFIG_PATH}
    if ! mount -t configfs none ${CONFIG_PATH} ; then
        printf "Unable to mount configfs\n"
        exit 1
    fi
    printf "Mounted configfs at %s\n" "${CONFIG_PATH}"
fi

# ----- Create a dummy device -----

if [ ! -d ${CONFIG_PATH}/iio/devices/dummy ]; then
    printf "No configfs path to create IIO devices.\n"
    exit 2
fi

DUMMY_CONFIG_PATH=${CONFIG_PATH}/iio/devices/dummy/${IIO_DEV_NAME}

if [ -d ${DUMMY_CONFIG_PATH} ]; then
    printf "IIO dummy device already exists.\n"
else
    mkdir ${DUMMY_CONFIG_PATH}
    printf "Created IIO dummy device: '%s'\n" "${IIO_DEV_NAME}"
fi

# ----- Dump the device info -----

printf "\n"
iio_info

