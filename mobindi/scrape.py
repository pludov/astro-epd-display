#!/usr/bin/env python3
# -*- coding: utf-8 -*-


# This python script run nmcli command to detect the wifi status
# It outputs a json object with the wifi status

import asyncio
import json
import sys
import time

wifi_memory = {}

async def get_wifi_status():
    # Wifi will be null or connection name
    wifi = None
    hotspot = None
    ethernet = None

    cmd = "nmcli --terse --fields uuid,type,name con show --active"
    process = await asyncio.create_subprocess_exec(*cmd.split(), stdout=asyncio.subprocess.PIPE)
    output, error = await process.communicate()
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
            if uuid in wifi_memory and wifi_memory[uuid]["cache_expiry_timestamp"] > time.time():
                dict = wifi_memory[uuid]
                # Clone the dict
                dict = dict.copy()
                dict.pop("cache_expiry_timestamp")
            else:
                process = await asyncio.create_subprocess_exec(*["nmcli", "--terse", "--show-secrets", "--fields", "802-11-wireless.ssid,802-11-wireless.mode,802-11-wireless-security.psk", "con", "show", "uuid", uuid], stdout=asyncio.subprocess.PIPE)
                # wifi-sec.psk
                output, error = await process.communicate()
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
                # Keep the value for 30 seconds min
                memorized = dict.copy()
                memorized["cache_expiry_timestamp"] = time.time() + 30
                wifi_memory[uuid] = memorized

            if dict["802-11-wireless.mode"] == "ap":
                hotspot = dict
            else:
                # Remove the psk
                if "802-11-wireless-security.psk" in dict:
                    dict.pop("802-11-wireless-security.psk")
                wifi = dict

        if type == "802-3-ethernet":
            ethernet = name

    return json.dumps({"wifi": wifi, "hotspot": hotspot, "ethernet": ethernet})


async def wifi_monitor():
    while True:
        process = await asyncio.create_subprocess_exec(*["nmcli", "monitor"], stdout=asyncio.subprocess.PIPE)
        while True:
            print(await get_wifi_status(), flush=True)
            res = await process.stdout.readline()
            if not res:
                break
        await process.wait()
        print(f"nmcli monitor exited with {process.returncode}", file=sys.stderr)
        # Sleep a few seconds before restarting the monitor
        await asyncio.sleep(2)


async def systemd_monitor():
    started_units = ["multi-user.target"]
    starting_units = [] # "local-fs.target"]
    stopping_units = ["reboot.target", "shutdown.target" ]
    status = ""

    all_units = started_units + starting_units + stopping_units
    while True:
        # Emit the running status according to:
        # , "--state=active", "--state=activating" "--state=inactive", 
        cmd = [ "systemctl", "list-units", "--quiet", "--full", "--plain", "--type=target", "--no-pager" ]
        # Capture the whole output
        process = await asyncio.create_subprocess_exec(*cmd, stdout=asyncio.subprocess.PIPE)
        output, error = await process.communicate()
        output = output.decode("latin-1")
        output = output.split("\n")
        active_units = {}
        activating_units = {}
        for entry in output:
            if entry == "":
                continue
            entry = entry.split(None, 3)
            if len(entry) != 4:
                print(f"Error parsing systemctl output {len(entry)}", file=sys.stderr)
                continue

            if entry[2] == "active":
                active_units[entry[0]] = True
            else:
                activating_units[entry[0]] = True
        
        new_status = ""
        for unit in started_units:
            if unit in active_units:
                new_status = "running"
                break
        for unit in starting_units + started_units:
            if unit in activating_units:
                new_status = "starting"
                break
        for unit in stopping_units:
            if unit in active_units:
                new_status = "stopping"
                break
            if unit in activating_units:
                new_status = "stopping"
                break
        if new_status != "" and new_status != status:
            status = new_status
            print(f"Status changed to {status}", file=sys.stderr)

            print(json.dumps({"sysstatus": status}), flush=True)

        await process.wait()

        # Sleep one second before emitting the status
        await asyncio.sleep(1)


async def main():

    tasks = [
        asyncio.create_task(wifi_monitor()),
        asyncio.create_task(systemd_monitor()),
    ]
    await asyncio.gather(*tasks)


if __name__ == '__main__':
    asyncio.run(main())