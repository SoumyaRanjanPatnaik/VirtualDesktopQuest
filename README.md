# Introduction

Virtual reality (VR) is an immersive technology that allows users to experience
and interact with simulated environments. Meta Quest headsets are one of the
most popular options for experiencing VR. Meta has created an application for
these headsets called “Meta Horizon Workrooms” that provides users with an
immersive environment where they can get things done. This app allows its users
to create and access virtual desktops, which are virtual screens connected to
their desktops or PCs. The aim of this project is to provide a Linux backend for
the Virtual Desktop tool that can work with the Meta Quest 2 headset. This will
allow Linux users to enjoy the benefits of VR productivity and gaming on their
computers. This project targets the following environment:

- Meta Quest 2
- sway, a tiling window manager based on i3, under the distribution Regolith
  Linux (which is a i3 flavor of Ubuntu)
- pipewire for audio capture

> Note: The work described in this post was done as part of my Google Summer of
> Code (GSoC) 2023 project under the organisation CCExtractor.

# Architecture

The Virtual Desktop tool consists of three main components: screen capture,
audio capture, and remote desktop. Screen capture is responsible for capturing
the frames from the user’s desktop or PC and sending them to the headset. Audio
capture is responsible for capturing the audio from the user’s microphone and
speakers and sending them to the headset. Remote desktop is responsible for
receiving the input events from the headset and sending them to the desktop or
PC.

Even though the initial target environment limits the usage to wayland-roots
based compositors, it would be wise to eventually add support for more display
backends. Similarly, we might want to support pipewire alternatives when it
comes to audio servers. To ensure future compatibility, I’ve created the traits
`AudioCaptureBackend` and `FrameCaptureBackend`. The `virtual_desktop::Manager`
will then inject the correct dependencies based on the session configuration.
When I have more info about the interface used by Horizon Workrooms, I’ll also
add an additional trait for Remote Desktop, in order to future proof the
application from potential breaking changes as well as keep open the possibility
of supporting other virtual desktop applications like Immersed (which currently
lacks support for wayland)

`AudioCaptureBackend` and `FrameCaptureBackend` run on their own seperate
threads and send the captured data to the `Virtual Desktop Server` using
channels. The synchronization of all three components of the virtual desktop
backend will be done by the main thread running the `virtual_desktop::Manager`.

# Implementation Details

The implementation of the Virtual Desktop tool is done in Rust, a systems
programming language that offers high performance, memory safety, and fearless
concurrency.

## Screen Capture

The `FrameCaptureBackend` provides `capture` which is used to capture from a
single output device. The capture can be of type `CaptureType::Frame` for
screenshots or `CaptureType::Stream` for screen recording.

For wayland, I use the wayland-client and wayland-protocols crates to interact
with the Wayland compositor and its protocols. The `WlrFramecaspturer`
implements `FrameCaptureBackend` for Wayland Roots based compositors (like sway)
which is our target environment. Specifically, I use the
`zwlr_screencopy_manager_v1` and `zwlr_screencopy_frame_v1` protocols to request
and receive frames from the output devices. The frames are written into a
`wl_buffer`, which represents a shared memory object. `zwlr_screencopy_frame_v1`
sends events notifying the application about the `DRM_FOURCC` format that is
used as well as other meta metadata required to properly encode the frames, like
the `width`, `height`, `stride`, etc. To test screencapture, the
`WlrFramecaspturer` currently converts the frames to `RGBA` and writes them into
a png file.

## Audio Capture

For audio capture, I use the `simple-pulse-desktop-capture` crate, which is a
wrapper for PulseAudio API. This crate allows me to create a stream for
recording audio from any source on the system. My initial plan was to use a
pipewire based implementation, but I couldn't find any proper documentation for
this. Moreover it seems like OBS (the benchmark for screenrecording and audio
capture) also uses `pulseaudio`. The audio data recieved is in the form of PCM
frames. I've tried writing this data to a wav file but the generated wav file
seems to be in an invalid format.

## Remote Desktop

This part requires a lot more information about Meta Horizon Workrooms client.
Most of the pieces are in place to allow for screen capture so progress would be
faster once I learn more about this.

# Current Status

The project is still a work in progress and nowhere near completion. However, I
have achieved some of the main goals and milestones of the project, such as:

- Implemented screen capture backend for sway
- Partially implmented audio capture
- Gained information on how remote desktop clients work
