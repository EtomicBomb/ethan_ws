resources-root []
    "/home/pi/Desktop/server/resources"

error-404-response []
    "HTTP/1.1 404 Page Not Found\r\n\r\n<!DOCTYPE html><html lang='en-US'><head><meta charset='UTF-8'><title>ethan.ws</title></head><body><h1>Error 404 - Page Not Found</h1></body></html>"

ok-response-header []
    "HTTP/1.1 200 OK\r\n\r\n"

http-handler [request]
    (let response-location (concat (resources-root) request)
        (let maybe-contents (read-file response-location)
            (if (= :file-not-found maybe-contents)
                (let with-index-suffix (read-file (print (concat response-location "index.html")))
                    (if (= :file-not-found with-index-suffix)
                        (error-404-response)
                        (concat (ok-response-header) with-index-suffix)))
                maybe-contents)))

