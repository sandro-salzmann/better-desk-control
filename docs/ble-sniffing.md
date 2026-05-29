# Sniffing the LINAK Desk Control app (BLE)

How to capture the Bluetooth traffic between the official Android app
(`com.linak.deskcontrol`) and the desk, so app actions can be mapped to GATT
writes. Findings go into [protocol.md](protocol.md).

## Prerequisites

- An Android phone with the LINAK app installed and paired with the desk, plus
  a USB cable.
- [Android Platform Tools](https://developer.android.com/tools/releases/platform-tools)
  (`adb`) on the PC.

## 1. Phone setup (one-time)

1. **Developer Options** — Settings → About phone → tap _Build number_ 7×.
2. **Bluetooth HCI snoop log** — Developer options → _Enable Bluetooth HCI snoop
   log_ → ON, then **toggle Bluetooth off and back on**. The log only starts
   recording after a Bluetooth restart; skip this and the capture comes back
   empty (the usual cause of an empty trace).
3. **USB debugging** — Developer options → _USB debugging_ → ON. Plug into the
   PC and accept the "Allow USB debugging" prompt.

## 2. Capture a session

Open the app, connect, and perform **one** action you want to understand (e.g.
"up for 2s", "go to preset 1", "save current height as preset 2"), noting the
wall-clock time so you can find it in the trace. One short, focused action per
capture is ideal.

## 3. Pull the log

```powershell
adb devices                 # confirm the phone is listed
adb bugreport btsnoop.zip   # includes the HCI log regardless of vendor
```

The log is inside the zip at `FS/data/misc/bluetooth/logs/btsnoop_hci.log`.
