# domx - HTML Parser and DOM builder

[![Status](https://img.shields.io/travis/hean01/domx/master.svg)](https://travis-ci.org/hean01/domx)

__domx__ includes a small HTML [Parser] and [DOM] builder for easing the
work with HTML data as structured data. The goal is to be very
resilience against invalid HTML documents, eg. missing closing tag
etc. In worst case you just get strange data from the parser.

The [Parser] itself runs through the HTML document and by using the
trait [IsParser], implemented by the caller as handler, you will be
notified when a opening tag, closing tag and data is parsed.
Information through the callback is provided as [Tag], a vector of
[Attribute] and data as a vector of u8. See example below how to use
the [Parser] and a simple implementation of [IsParser].


The [DOM] builder uses the parser to build up a tree data
structure of the HTML document. Which you can traverse and perform
operations on such as cleaning up the document or just simplify
it. Running a broken HTML, eg missing closing tags, into DOM and
then saving it you will get a nice consistent and valid HTML file.

__domx__ is licensed under GPLv3
