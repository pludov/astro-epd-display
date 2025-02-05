#!/usr/bin/env python3
# -*- coding: utf-8 -*-


# This python script run nmcli command to detect the wifi status
# It outputs a json object with the wifi status

from multiprocessing import Process, connection
import subprocess
import json
import sys
import time

def get_wifi_status():
    # Wifi will be null or connection name
    wifi = None
    hotspot = None
    ethernet = None

    cmd = "nmcli --terse --fields uuid,type,name con show --active"
    process = subprocess.Popen(cmd.split(), stdout=subprocess.PIPE)
    output, error = process.communicate()
    output = output.decode("utf-8")
    # Split the output by newline
    output = output.split("\n")
    # Each entry is an active connection. Identify ethernet & 802-11-wireless ones
    for entry in output:
        if entry == "":
            continue
        # Each entry is in the format "type,name"
        entry = entry.split(":")
        if len(entry) != 3:
            # Print to stderr
            print("Error parsing nmcli output", file=sys.stderr)
            continue
        (uuid, type, name) = entry
        if type == "802-11-wireless":
            # Extract the 802-11-wireless.mode, ssid, wifi-sec.psk
            process = subprocess.Popen(["nmcli", "--terse", "--show-secrets", "--fields", "802-11-wireless.ssid,802-11-wireless.mode,802-11-wireless-security.psk", "con", "show", "uuid", uuid], stdout=subprocess.PIPE)
            # wifi-sec.psk
            output, error = process.communicate()
            output = output.decode("utf-8")
            output = output.split("\n")
            dict = {}
            for entry in output:
                if entry == "":
                    continue
                entry = entry.split(":", 1)
                if len(entry) != 2:
                    print("Error parsing nmcli output", file=sys.stderr)
                    continue
                (key, value) = entry
                dict[key] = value

            if dict["802-11-wireless.mode"] == "ap":
                hotspot = dict
            else:
                # Remove the psk
                dict.pop("802-11-wireless-security.psk")
                wifi = dict

        if type == "802-3-ethernet":
            ethernet = name

    return json.dumps({"wifi": wifi, "hotspot": hotspot, "ethernet": ethernet})


def wifi_monitor():
    process = subprocess.Popen(["nmcli", "monitor"], stdout=subprocess.PIPE)
    while True:
        print(get_wifi_status(), flush=True)
        process.stdout.readline()


def systemd_monitor():
    started_units = ["multi-user.target"]
    starting_units = ["local-fs.target"]

    status = ""

    all_units = started_units + starting_units
    while True:
        # Emit the running status according to:
        cmd = [ "systemctl", "list-units", "--quiet", "--full", "--plain", "--state=active", "--type=target", "--no-pager" ] + all_units
        # Capture the whole output
        process = subprocess.Popen(cmd, stdout=subprocess.PIPE)
        output, error = process.communicate()
        output = output.decode("latin-1")
        output = output.split("\n")
        active_units = {}
        for entry in output:
            if entry == "":
                continue
            entry = entry.split(" ", 1)
            if len(entry) != 2:
                print(f"Error parsing systemctl output {len(entry)}", file=sys.stderr)
                continue
            active_units[entry[0]] = True
        print(json.dumps({"active_units": active_units}), flush=True)
        new_status = ""
        for unit in starting_units:
            if unit in active_units:
                new_status = "starting"
                break
        for unit in started_units:
            if unit in active_units:
                new_status = "running"
                break
        if new_status != status:
            status = new_status
            print(json.dumps({"status": status}), flush=True)

        process.wait()

        # Sleep one second before emitting the status
        time.sleep(1)

if __name__ == '__main__':
    
    tasks = [
        Process(target=wifi_monitor),
        Process(target=systemd_monitor),
    ]
    for p in tasks:
        p.daemon = True
        p.start()
    
    # Wait the first process to finish
    connection.wait(p.sentinel for p in tasks)

