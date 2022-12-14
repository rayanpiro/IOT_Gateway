# Tipos de datos.

1.  Datos peticion recurrente, frontend envia el ciclo de scan de esos datos y el hardware publica sin petición los mismos.
En caso de no conexión se almacenan localmente (a definir) para su posterior envio por lotes.
2.  Datos peticion bajo demanda. Frontend envia la peticion de un dato y el hardware responde el dato.
En caso de no conexión el dato nunca llega a destino.
3.  Eventos. Hardware monitoriza el cambio de estado de un dato y lo notifica a la plataforma.
En caso de no conexión se almacenan localmente (a definir) para su posterior envio por lotes.
4.  Comandos:
    - Ping: Realiza una lectura. Si es satisfactoria devuelve un PONG del device, si no devuelve el error al cabo de un TIMEOUT_S.
    - Read: Realiza una lectura. Si es satisfactoria devuelve un dato, si no devuelve el error.
    - Write \<dato\>: Realiza una escritura del dato pasado como parametro en el tag elegido.

# Arbol de directorios.

    /protocol/device_name/
                        ./connection.ini   -> Datos de conexión al dispositivo.
                        ./subscribers.ini  -> Comandos de escritura desde el broker.
                        ./publishers.ini   -> Lectura de datos que se publican en el broker MQTT.
                        ./events.ini       -> Datos a monitorizar en local para notificar sólo los cambios de estado, no en continuo.

Ejemplo de connection.ini para protocolo modbus tcp.

    [CONNECTION_PARAMETERS]
    ip=10.19.8.60
    port=1442
    slave=31

Ejemplo de publishers.ini para protocolo modbus tcp.

    [Tension_R]
    address=7
    length=2
    command=ReadHolding
    swap=BigEndian
    data_type=Float

    [Tension_S]
    address=9
    length=2
    command=ReadHolding
    swap=BigEndian
    data_type=Float

    [Tension_T]
    address=11
    length=2
    command=ReadHolding
    swap=BigEndian
    data_type=Float

# Estructura MQTT.

    /client_id/warehouse_id/
                            /measures/{device_id}/{tag_name}  -> Publicación de las medidas sin petición.
                            /events/{device_id}/{tag_name}    -> Publicación de cambios de estado sin petición.
                            /commands/{device_id}/{tag_name}  -> Envio de comandos de escritura, peticion de lectura, PING request.

# Estructura del codigo.
1. Leer la carpeta config que tendrá a su vez una carpeta por cada protocolo.
2. Leer de forma recursiva las carpetas device_id con los ficheros correspondientes a cada protocolo. Necesario un ini_parser por cada protocolo.
3. Cargar el Vec<Tag> en memoria, que contiene lo necesario para identificar un tag (su name), para conectarse (sus connection properties), y para leerlo o escribirlo (su address) propios de cada protocolo.
4. Iniciar un bucle de lectura tanto de los tags como de los commands recibidos por el MQTT broker.

# TODOS
- Consensuar los mensajes de MQTT con David.
- Probar las escrituras de Modbus TCP y Modbus RTU.
- Hacer los tags dependientes del device. Entendiendo un device como un solo device (no una pasarela modbus rtu over ethernet). Para que un read al device devuelva el JSON de todos los tags del mismo.
- Generar eventos.
- Revisar los unwrap del codigo.
- Integrar más test.
