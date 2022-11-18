APPLICATION = wot-coap-generation
BOARD ?= native
RIOTBASE ?= $(CURDIR)/../../riotos
ORIGINAL_CODE = $(RIOTBASE)/tests/wot_coap_generation

APPLICATION_RUST_MODULE = wot_coap_generation
BASELIBS += $(APPLICATION_RUST_MODULE).module

FEATURES_REQUIRED += rust_target

BOARD_BLACKLIST := ek-lm4f120xl

# CARGO_EXTRACFLAGS +=

# WoT
USEMODULE += wot_core
USEMODULE += wot_coap
USEMODULE += wot_coap_config
USEMODULE += xtimer

WOT_THING_MODELS += config/wot_td/thing_model.json
WOT_META_DATA := config/wot_td/meta_data.json
WOT_BINDINGS_PATH := config/wot_td/bindings.json
WOT_PLACEHOLDERS_PATH := config/wot_td/placeholders.json

ifeq ($(BOARD),esp32-wroom-32)
# ESP wifi configuration
USEMODULE += esp_wifi
WIFI_SSID?=\"Your\ SSID\"
WIFI_PASS?=\"Your\ Password\"
CFLAGS += -DESP_WIFI_SSID=$(WIFI_SSID)
CFLAGS += -DESP_WIFI_PASS=$(WIFI_PASS)

# GPIO Bindings
LED_GPIO = GPIO33
CFLAGS += -DLED_GPIO=$(LED_GPIO)
endif

ifeq ($(BOARD),native)
CFLAGS += -DIS_NATIVE
endif

# Shell
# USEMODULE += shell
# USEMODULE += shell_commands
# USEMODULE += ps

# Include packages that pull up and auto-init the link layer.
# NOTE: 6LoWPAN will be included if IEEE802.15.4 devices are present
USEMODULE += gnrc_netdev_default
USEMODULE += auto_init_gnrc_netif
# Activate ICMPv6 error messages
USEMODULE += gnrc_icmpv6_error
USEMODULE += gnrc_icmpv6_echo
# Specify the mandatory networking modules for IPv6 and UDP
USEMODULE += gnrc_ipv6_default
USEMODULE += gnrc_udp
# Add a routing protocol
USEMODULE += gnrc_rpl
USEMODULE += auto_init_gnrc_rpl
# This application dumps received packets to STDIO using the pktdump module
USEMODULE += gnrc_pktdump
# Additional networking modules that can be dropped if not needed
USEMODULE += gnrc_icmpv6_echo
USEMODULE += netstats_l2
USEMODULE += netstats_ipv6
USEMODULE += netstats_rpl

include $(RIOTBASE)/Makefile.include