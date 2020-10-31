#!/bin/bash
#
# load_dummy.sh
#
# Loads the required IIO kernel modules and then creates a dummy device
# and one hrtimer.
# 
# This is useful for application testing and unit testing of the library.
# 
# The required IIO drivers must be comiled for the kernel as loadable modules.
# This should be the case for Ubuntu 18.04 and Mint 19. Your mileage may vary.
#

# Names of the device and timer that will be created
IIO_DEV_NAME=dummydev
IIO_TIMER_NAME=timer0

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

HRTIMER_LOADED=$(lsmod | grep ^iio_trig_hrtimer)

if [ -n "${HRTIMER_LOADED}" ]; then
    printf "IIO hrtimer module already loaded.\n"
else
    if ! modprobe iio_trig_hrtimer ; then
        printf "Unable to load load the IIO hrtimer module.\n"
    fi
    printf "IIO hrtimer module loaded.\n"
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

IIO_CONFIG_PATH=${CONFIG_PATH}/iio

# ----- Create a dummy device -----

if [ ! -d ${IIO_CONFIG_PATH}/devices/dummy ]; then
    printf "No configfs path to create IIO devices.\n"
    exit 2
fi

DEV_CONFIG_PATH=${IIO_CONFIG_PATH}/devices/dummy/${IIO_DEV_NAME}

if [ -d ${DEV_CONFIG_PATH} ]; then
    printf "IIO dummy device already exists.\n"
else
    mkdir ${DEV_CONFIG_PATH}
    printf "Created IIO dummy device: '%s'\n" "${IIO_DEV_NAME}"
fi

# ----- Create a timer -----

if [ ! -d ${IIO_CONFIG_PATH}/triggers/hrtimer ]; then
    printf "No configfs path to create an IIO hrtimer.\n"
else
    TIMER_CONFIG_PATH=${IIO_CONFIG_PATH}/triggers/hrtimer/${IIO_TIMER_NAME}

    if [ -d ${TIMER_CONFIG_PATH} ]; then
        printf "IIO hrtimer already exists.\n"
    else
        mkdir ${TIMER_CONFIG_PATH}
        printf "Created IIO hrtimer: '%s'\n" "${IIO_TIMER_NAME}"
    fi
fi

# ----- Dump the device info -----

printf "\n--- IIO Info ---\n\n"
iio_info

