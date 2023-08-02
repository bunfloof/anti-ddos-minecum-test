# Anti-DDoS test stuff (Minecum)

GRRRRRRR WOOF ARF AWOO RRRR ARF WOOF BARK WOOF GRRRR BARK GRRRRRRR BARK SNARL ARF GROWL GROWL ARF SNARL ARRF SNARL WOOF GRRRRR RUFF WOOF AWWOOOOOOOO AWOOOO BARK ARF WOOF BARK RRRRRRR BARK GRRRR WOOF WOOF RUFF GRRRRRR WOOF

DDoS me = I coem 🤑🤤🥵

skid: DDoSes me

me: *cums* 😩

## Notes

- For VampSMP, my favorite furry Minecraft server, that keeps on getting DDoSed by some kid. 

- These are only experimental snippets intended for use under protected upstreams. Tested with: Path.net and Voxility.

### Attack Report on 07-31-2023
The client connects to the server, sends a "Ping" packet, sends a "Handshake" packet with a protocol version of 2 (which is invalid since Minecraft version 1.20.1 uses 763), sends a "Login" packet with a strange username, and the client closes the connection.

I temporarily enabled passthrough on the server's firewall to capture this attack.

Sample (filtered):
```
[2023-08-01 00:00:00] INFO - Accepted connection from 192.168.1.1:12345
[2023-08-01 00:00:00] INFO - Received ping packet from 192.168.1.1:12345
[2023-08-01 00:00:00] INFO - Received handshake packet with protocol version 2 from 192.168.1.1:12345
[2023-08-01 00:00:00] ERROR - Invalid handshake version from 192.168.1.1:12345. Disconnecting client.
[2023-08-01 00:00:00] INFO - Accepted connection from 192.168.1.2:12346
[2023-08-01 00:00:00] INFO - Received ping packet from 192.168.1.2:12346
[2023-08-01 00:00:00] INFO - Received login packet with username " " from 192.168.1.2:12346
[2023-08-01 00:00:00] ERROR - Invalid username from 192.168.1.2:12346. Disconnecting client.
[2023-08-01 00:00:00] WARN - High volume of requests detected: 50 requests in the last second
...
[2023-08-01 00:00:05] WARN - High volume of requests detected: 300 requests in the last second
[2023-08-01 00:00:05] INFO - Accepted connection from 192.168.1.255:12495
[2023-08-01 00:00:05] INFO - Received ping packet from 192.168.1.255:12495
[2023-08-01 00:00:05] INFO - Received login packet with long username from 192.168.1.255:12495
[2023-08-01 00:00:05] ERROR - Invalid username from 192.168.1.255:12495. Disconnecting client.
...
[2023-08-01 00:00:06] WARN - High volume of requests detected: 300 requests in the last second
[2023-08-01 00:00:06] ERROR - Too many requests. Limiting new connections.
...
[2023-08-01 00:00:10] WARN - High volume of requests detected: 300 requests in the last second
[2023-08-01 00:00:10] ERROR - Too many requests. Limiting new connections.
```

### Attack Report on 08-01-2023

The attacker appears to be using a bot to rapidly create and drop connections, each sending a "Handshake" packet and a "Login" packet with a specific, human, constant username. Still, they're using the wrong protocol version for attacking a Minecraft version 1.20.1 server.

Sample (filtered):
```
[2023-08-02 00:00:00] INFO - Accepted connection from 192.168.1.1:12345
[2023-08-02 00:00:00] INFO - Received handshake packet with protocol version 59 from 192.168.1.1:12345
[2023-08-02 00:00:00] ERROR - Invalid handshake version from 192.168.1.1:12345. Disconnecting client.
[2023-08-02 00:00:00] INFO - Accepted connection from 192.168.1.2:12346
[2023-08-02 00:00:00] INFO - Received login packet with username "fuckyou" from 192.168.1.2:12346
[2023-08-02 00:00:00] WARN - High volume of requests detected: 50 requests in the last second
...
[2023-08-02 00:00:05] WARN - High volume of requests detected: 300 requests in the last second
[2023-08-02 00:00:05] INFO - Accepted connection from 192.168.1.255:12495
[2023-08-02 00:00:05] INFO - Received handshake packet with protocol version 59 from 192.168.1.255:12495
[2023-08-02 00:00:05] ERROR - Invalid handshake version from 192.168.1.255:12495. Disconnecting client.
...
[2023-08-02 00:00:06] WARN - High volume of requests detected: 300 requests in the last second
[2023-08-02 00:00:06] ERROR - Too many requests. Limiting new connections.
...
[2023-08-02 00:00:10] WARN - High volume of requests detected: 300 requests in the last second
[2023-08-02 00:00:10] ERROR - Too many requests. Limiting new connections.
```