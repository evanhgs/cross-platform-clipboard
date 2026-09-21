## Releases Linux

```bash
git tag v0.1.0
git push origin v0.1.0
```

To use : (`chmod +x`), then open

Build locally, it's easy thanks to dioxus 

```bash
dx bundle --desktop --package-types deb --package-types appimage
```

OUTPUT are on `target/dx/.../bundle/`.

notes for me... Never use `dx fmt` it breaks code, yeah that suck, i wish i was clever to analyse and debug this, but the reality is i am lazy and i will skip the issue, great. 